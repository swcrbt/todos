// tray-todo 原型交互逻辑（对应真实前端 app.mjs 的契约；仅原型演示使用）
// 数据访问仅经 mock-data.mjs（对应 RUST todo-store 的 command 层语义）。

import * as store from "./mock-data.mjs";
import { isValidInput, formatCount } from "./logic.mjs";

const card = document.getElementById("card");
const inputEl = document.getElementById("todo-input");
const listEl = document.getElementById("todo-list");
const emptyEl = document.getElementById("empty-state");
const countEl = document.getElementById("count-badge");
const bannerEl = document.getElementById("error-banner");
const trayIcon = document.getElementById("tray-icon");
const failBtn = document.getElementById("simulate-fail");
const failOffBtn = document.getElementById("simulate-fail-off");
const noteEl = document.getElementById("proto-note");

let bannerTimer = null;

/* ---------- 渲染 ---------- */

function render() {
  const items = store.list();
  countEl.textContent = formatCount(items.length);
  emptyEl.hidden = items.length > 0;
  listEl.replaceChildren(
    ...items.map((item) => {
      const li = document.createElement("li");
      li.className = "todo-item" + (item.done ? " done" : "");
      li.dataset.id = item.id;

      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";
      checkbox.checked = item.done;
      checkbox.setAttribute("aria-label", item.done ? `取消完成：${item.text}` : `标记完成：${item.text}`);
      checkbox.addEventListener("change", () => onToggle(item.id, checkbox));

      const text = document.createElement("span");
      text.className = "todo-text";
      text.textContent = item.text;

      const del = document.createElement("button");
      del.type = "button";
      del.className = "delete-btn";
      del.textContent = "\u00d7"; // 关闭符号
      del.setAttribute("aria-label", `删除：${item.text}`);
      del.addEventListener("click", (event) => {
        event.stopPropagation(); // 防止 render 重建 DOM 后冒泡被误判为外部点击
        onRemove(item.id);
      });

      li.append(checkbox, text, del);
      return li;
    })
  );
}

/* ---------- 交互（对应 INT-001 command 语义） ---------- */

function onAdd() {
  const text = inputEl.value;
  if (!isValidInput(text)) {
    return; // 空输入不添加（SCN-TODO-001-B）
  }
  try {
    store.add(text);
    inputEl.value = "";
  } catch (err) {
    showBanner(`保存失败：${err.message}`);
  }
}

function onToggle(id, checkbox) {
  try {
    store.toggle(id);
  } catch (err) {
    checkbox.checked = !checkbox.checked; // 失败时还原控件状态
    showBanner(`保存失败：${err.message}`);
  }
}

function onRemove(id) {
  try {
    store.remove(id);
  } catch (err) {
    showBanner(`保存失败：${err.message}`);
  }
}

function showBanner(message) {
  bannerEl.textContent = message;
  bannerEl.hidden = false;
  clearTimeout(bannerTimer);
  bannerTimer = setTimeout(() => { bannerEl.hidden = true; }, 3000);
}

/* ---------- 窗口显隐（对应 INT-002/INT-003 模拟） ---------- */

function toggleCard() {
  if (card.hidden) {
    card.hidden = false;
    inputEl.focus();
  } else {
    hideCard();
  }
}

function hideCard() {
  card.hidden = true;
  inputEl.value = ""; // 失焦关闭时未提交输入丢弃（INV-007；真实实现可在 hide 时清空）
  bannerEl.hidden = true;
}

/* ---------- 事件绑定 ---------- */

inputEl.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    onAdd();
  }
});

// 模拟托盘图标：点击切换卡片（SCN-TRAY-002-A/B）
trayIcon.addEventListener("click", (event) => {
  event.stopPropagation(); // 阻止触发下方"外部点击隐藏"
  toggleCard();
});

// 模拟失焦：点击卡片外部区域 → 自动隐藏（SCN-TRAY-003-A）
document.addEventListener("click", (event) => {
  if (!card.hidden && !card.contains(event.target)) {
    hideCard();
  }
});

// 原型工具：模拟保存失败（受限状态演示；真实运行路径无此入口）
failBtn.addEventListener("click", (event) => {
  event.stopPropagation();
  store.setPersistFail(true);
  failBtn.hidden = true;
  failOffBtn.hidden = false;
  noteEl.textContent = "持久化模拟已开启：任何变更都会失败并提示错误横幅";
});

failOffBtn.addEventListener("click", (event) => {
  event.stopPropagation();
  store.setPersistFail(false);
  failOffBtn.hidden = true;
  failBtn.hidden = false;
  noteEl.textContent = "";
});

store.subscribe(render);

/* ---------- 原型验证钩子（仅原型调试/自动化断言使用） ---------- */
window.__pt = {
  items: () => JSON.parse(JSON.stringify(store.__protoDebug.getItems())),
  cardHidden: () => card.hidden,
  bannerHidden: () => bannerEl.hidden,
  bannerText: () => bannerEl.textContent,
  inputValue: () => inputEl.value,
  count: () => store.list().length,
};

// 初始渲染（若 localStorage 已有数据则直接恢复 → 模拟重启恢复）
render();