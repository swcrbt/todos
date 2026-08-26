// tray-todo 原型真实 UI 验证驱动（CDP）
// 运行前提：
//   1) python3 -m http.server 8701 --directory <本仓库>/openspec/changes/tray-todo/prototype
//   2) Chrome headless："/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
//        --headless=new --remote-debugging-port=9333 --user-data-dir=<tmp> \
//        --no-first-run --disable-gpu --window-size=1280,800 about:blank
//   3) node tools/validate-ui.mjs
// 输出：屏幕逐步通过/失败摘要 + screenshots/*.png（UI Evidence）
import { writeFileSync, mkdirSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const PROTO_URL = process.env.PROTO_URL || "http://127.0.0.1:8701/index.html";
const DEBUG_PORT = process.env.DEBUG_PORT || "9333";
const SHOT_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "../screenshots");

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---------- CDP 会话 ----------
let msgId = 0;
const pending = new Map();
const events = { consoleErrors: [], exceptions: [], fatalErrors: [] };
let ws;

function send(method, params = {}) {
  return new Promise((resolveFn, rejectFn) => {
    const id = ++msgId;
    pending.set(id, { resolveFn, rejectFn });
    ws.send(JSON.stringify({ id, method, params }));
  });
}

async function connect() {
  const targets = await (await fetch(`http://127.0.0.1:${DEBUG_PORT}/json/list`)).json();
  const page = targets.find((t) => t.type === "page");
  if (!page) throw new Error("未找到 Chrome 页面目标");
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res) => (ws.onopen = res));
  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) {
      const { resolveFn, rejectFn } = pending.get(msg.id);
      pending.delete(msg.id);
      if (msg.error) rejectFn(new Error(`${msg.error.message}`));
      else resolveFn(msg.result);
    } else if (msg.method === "Runtime.consoleAPICalled") {
      if (msg.params.type === "error") {
        events.consoleErrors.push(msg.params.args.map((a) => a.value ?? a.description ?? "").join(" "));
      }
    } else if (msg.method === "Runtime.exceptionThrown") {
      events.exceptions.push(msg.params.exceptionDetails.text);
    } else if (msg.method === "Log.entryAdded" && msg.params.entry.level === "error") {
      events.fatalErrors.push(msg.params.entry.text);
    }
  };
  await send("Page.enable");
  await send("Runtime.enable");
  await send("Log.enable");
}

function methodName() {
  return "";
}

// ---------- 辅助 ----------
async function evalJs(expression) {
  const res = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (res.exceptionDetails) throw new Error(`JS 执行异常: ${res.exceptionDetails.text}`);
  return res.result.value;
}

async function rect(selector) {
  return evalJs(`(() => { const el = document.querySelector(${JSON.stringify(selector)}); if (!el) return null; const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height, cx: r.x + r.width / 2, cy: r.y + r.height / 2 }; })()`);
}

async function mouse(type, x, y) {
  await send("Input.dispatchMouseEvent", { type, x, y, button: "left", clickCount: type === "mouseMoved" ? 0 : 1 });
}

async function clickAt(x, y) {
  await mouse("mouseMoved", x, y);
  await mouse("mousePressed", x, y);
  await mouse("mouseReleased", x, y);
}

async function pressEnter() {
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13, nativeVirtualKeyCode: 13 });
  await send("Input.dispatchKeyEvent", { type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13, nativeVirtualKeyCode: 13 });
}

async function typeText(text) {
  await evalJs(`(() => { const el = document.querySelector("#todo-input"); el.value = ${JSON.stringify(text)}; el.dispatchEvent(new Event("input", { bubbles: true })); el.focus(); })()`);
}

async function snapshot() {
  return evalJs(`(() => {
    const items = [...document.querySelectorAll(".todo-item")];
    const vis = (el) => { if (!el) return null; const cs = getComputedStyle(el); return { display: cs.display, opacity: cs.opacity, color: cs.color, lineThrough: cs.textDecorationLine }; };
    return {
      cardHidden: document.getElementById("card").hidden,
      count: items.length,
      items: items.map((li) => ({
        text: li.querySelector(".todo-text").textContent,
        done: li.classList.contains("done"),
        checkboxChecked: li.querySelector("input[type=checkbox]").checked,
        deleteOpacity: vis(li.querySelector(".delete-btn")).opacity,
        textDecoration: vis(li.querySelector(".todo-text")).lineThrough,
        textColor: vis(li.querySelector(".todo-text")).color,
      })),
      emptyVisible: !document.getElementById("empty-state").hidden,
      bannerHidden: document.getElementById("error-banner").hidden,
      bannerText: document.getElementById("error-banner").textContent,
      inputValue: document.getElementById("todo-input").value,
    };
  })()`);
}

function buf(data) {
  return Buffer.from(data, "base64");
}

async function shot(name) {
  mkdirSync(SHOT_DIR, { recursive: true });
  const clip = await rect("#card");
  const full = await send("Page.captureScreenshot", { format: "png" });
  writeFileSync(join(SHOT_DIR, `${name}.png`), buf(full.data));
  // 卡片可见时才额外保存卡片级特写（隐藏态截页面级即可）
  if (clip && clip.w > 0 && clip.h > 0) {
    const c = await send("Page.captureScreenshot", { format: "png", clip: { x: clip.x, y: clip.y, width: clip.w, height: clip.h, scale: 1 } });
    writeFileSync(join(SHOT_DIR, `${name}-card.png`), buf(c.data));
  }
  return `${name}.png`;
}

// ---------- 结果记录 ----------
const results = [];
function record(step, pass, detail = "") {
  results.push({ step, pass, detail });
  console.log(`${pass ? "PASS" : "FAIL"} | ${step}${detail ? " | " + detail : ""}`);
}

async function assertEq(step, actual, expected, label = "值") {
  const pass = actual === expected;
  record(step, pass, `${label}: ${JSON.stringify(actual)} (期望 ${JSON.stringify(expected)})`);
  return pass;
}

// ---------- 主流程 ----------
async function main() {
  await connect();

  // 打开页面并重置原型 mock 数据（保证从干净空态开始验证）
  await send("Page.navigate", { url: PROTO_URL });
  await sleep(600);
  await evalJs(`localStorage.removeItem("trayTodo.prototype.v1")`);
  await send("Page.reload", { ignoreCache: true });
  await sleep(700);
  let s = await snapshot();
  record("初始-卡片隐藏", s.cardHidden === true, `cardHidden=${s.cardHidden}`);
  record("初始-空列表", s.count === 0, `count=${s.count}`);

  // SCN-TRAY-002-A：隐藏时点击托盘图标 → 显示
  let tray = await rect("#tray-icon");
  await clickAt(tray.cx, tray.cy);
  await sleep(350);
  s = await snapshot();
  const pass1 = s.cardHidden === false && s.emptyVisible === true;
  record("打开卡片且空态可见", pass1, `cardHidden=${s.cardHidden}, empty=${s.emptyVisible}`);
  await shot("01-empty");

  // 模拟手动预览可见性：输入框获取焦点
  await evalJs(`document.querySelector("#todo-input").focus()`);
  record("显示后输入框可聚焦", await evalJs(`document.activeElement === document.querySelector("#todo-input")`));

  // SCN-TODO-001-B：空输入回车不添加
  await typeText("");
  await pressEnter();
  await sleep(150);
  s = await snapshot();
  record("空输入回车不添加", s.count === 0 && s.inputValue === "", `count=${s.count}, input="${s.inputValue}"`);

  // SCN-TODO-001-A：依次添加 3 条，顺序 = 添加顺序，输入框清空
  for (const text of ["买牛奶", "开会", "写周报"]) {
    await typeText(text);
    await pressEnter();
    await sleep(150);
  }
  s = await snapshot();
  const texts = s.items.map((i) => i.text);
  const pass2 = s.count === 3 && s.inputValue === "" && texts.join("|") === "买牛奶|开会|写周报";
  record("添加3条且顺序正确、输入框清空", pass2, `items=${texts.join("|")}`);
  record("完成状态默认 pending", s.items.every((i) => !i.done));
  await shot("02-added");

  // SCN-TODO-003-A：悬停第 2 条显示删除按钮，其余不显示
  let li2 = await rect(".todo-item:nth-child(2)");
  await mouse("mouseMoved", li2.cx, li2.cy);
  await sleep(250);
  s = await snapshot();
  const delVis = s.items.map((i) => i.deleteOpacity);
  record("悬停仅显示被悬停条目的删除按钮", delVis[1] === "1" && delVis[0] === "0" && delVis[2] === "0", `opacities=${delVis.join(",")}`);
  await shot("03-hover-delete");

  // SCN-TODO-002-A：勾选第 1 条 → 划线置灰保留
  let cb1 = await rect(".todo-item:nth-child(1) input[type=checkbox]");
  await clickAt(cb1.cx, cb1.cy);
  await sleep(200);
  s = await snapshot();
  const item1 = s.items[0];
  record("勾选完成-状态", s.count === 3 && item1.done === true && item1.checkboxChecked === true, `count=${s.count}`);
  record("勾选完成-划线置灰", item1.textDecoration && item1.textDecoration.includes("line-through") && item1.textColor !== "rgb(31, 35, 40)", `decoration=${item1.textDecoration}, color=${item1.textColor}`);
  record("勾选完成-保留在列表", s.items.map((i) => i.text).join("|") === "买牛奶|开会|写周报");
  await shot("04-checked");

  // SCN-TODO-002-B：再次点击 → 恢复 pending
  await clickAt(cb1.cx, cb1.cy);
  await sleep(200);
  s = await snapshot();
  record("取消勾选恢复", s.items[0].done === false && s.items[0].textDecoration === "none", `decoration=${s.items[0].textDecoration}`);
  await shot("05-unchecked");

  // 重新勾选以验证删除完成条目
  await clickAt(cb1.cx, cb1.cy);
  await sleep(150);

  // SCN-TODO-003-B：悬停"开会"并点击删除（无确认）
  await mouse("mouseMoved", li2.cx, li2.cy);
  await sleep(200);
  let del2 = await rect(".todo-item:nth-child(2) .delete-btn");
  await clickAt(del2.cx, del2.cy);
  await sleep(250);
  s = await snapshot();
  record("删除后条目消失且无确认", s.count === 2 && !s.items.some((i) => i.text === "开会") && s.items[0].done === true, `count=${s.count}, texts=${s.items.map((i) => i.text).join("|")}`);
  await shot("06-after-delete");

  // SCN-TRAY-003-A：点击卡片外部 → 自动隐藏
  await clickAt(200, 700);
  await sleep(300);
  s = await snapshot();
  record("点击外部自动隐藏", s.cardHidden === true, `cardHidden=${s.cardHidden}`);
  await shot("07-hidden");

  // SCN-TRAY-002-A：再次点击托盘图标 → 重新显示
  tray = await rect("#tray-icon");
  await clickAt(tray.cx, tray.cy);
  await sleep(350);
  s = await snapshot();
  record("再次点击托盘重新显示", s.cardHidden === false && s.count === 2, `cardHidden=${s.cardHidden}, count=${s.count}`);
  await shot("08-reopened");

  // SCN-TODO-004-A：reload（模拟重启）→ 列表与完成状态恢复
  await send("Page.reload", { ignoreCache: true });
  await sleep(700);
  s = await snapshot();
  const restoredOk = s.count === 2 && s.items.map((i) => i.text).join("|") === "买牛奶|写周报" && s.items[0].done === true && s.items[1].done === false;
  record("reload后列表与完成状态一致", restoredOk, `items=${s.items.map((i) => `${i.text}:${i.done ? "done" : "pending"}`).join("|")}`);
  await shot("09-reload-restored");

  // 受限状态：模拟保存失败 → 错误横幅；失败不改数据
  let tray2 = await rect("#tray-icon");
  await clickAt(tray2.cx, tray2.cy);
  await sleep(350);
  let failBtn = await rect("#simulate-fail");
  await clickAt(failBtn.cx, failBtn.cy);
  await sleep(200);
  await typeText("测试");
  await pressEnter();
  await sleep(200);
  s = await snapshot();
  record("保存失败-不提交变更", s.count === 2, `count=${s.count}`);
  record("保存失败-错误横幅出现", s.bannerHidden === false && s.bannerText.includes("保存失败"), `banner="${s.bannerText}"`);
  await shot("10-error-banner");

  // 恢复正常写盘并成功添加一条验证恢复
  let offBtn = await rect("#simulate-fail-off");
  await clickAt(offBtn.cx, offBtn.cy);
  await sleep(150);
  await typeText("新任务");
  await pressEnter();
  await sleep(200);
  s = await snapshot();
  record("恢复后添加成功", s.count === 3 && s.items[2].text === "新任务", `count=${s.count}`);
  await shot("11-recovered");

  // 清理演示数据：reload 会保留（演示持久化）；不清理，原型工具负责 reset。
  // 最终 console/异常汇总
  const errors = {
    consoleErrors: events.consoleErrors,
    exceptions: events.exceptions,
    logErrors: events.fatalErrors,
  };
  record("无运行时错误", errors.consoleErrors.length === 0 && errors.exceptions.length === 0, JSON.stringify(errors));

  console.log("\n=== UI VALIDATION SUMMARY ===");
  console.log(JSON.stringify({ results, errors, screenshots: ["01~11 全量页面级 + 卡片级截图", SHOT_DIR] }, null, 2));
  const failed = results.filter((r) => !r.pass);
  process.exit(failed.length ? 1 : 0);
}

main().catch((err) => {
  console.error("驱动失败:", err);
  process.exit(2);
});