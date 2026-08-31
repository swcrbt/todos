// 卡片前端主逻辑：列表渲染、添加、勾选、悬停删除、错误横幅与空状态。
// 数据访问仅经 Tauri command API（后端 todo-store），前端不持有持久化状态。

import { isValidInput, formatCount } from "./logic.mjs";

// 全局 Tauri API（应用启用了 withGlobalTauri）
const invoke = window.__TAURI__.core.invoke;
const tauriEvent = window.__TAURI__.event;

const inputEl = document.getElementById("todo-input");
const listEl = document.getElementById("todo-list");
const emptyEl = document.getElementById("empty-state");
const countEl = document.getElementById("count-badge");
const bannerEl = document.getElementById("error-banner");

let bannerTimer = null;

/** 展示中文错误横幅，3 秒后自动消失 */
function showBanner(message) {
  bannerEl.textContent = message;
  bannerEl.hidden = false;
  clearTimeout(bannerTimer);
  bannerTimer = setTimeout(() => { bannerEl.hidden = true; }, 3000);
}

/** 全量渲染列表：计数徽标、空状态、条目列表（顺序 = 后端返回顺序） */
function render(items) {
  countEl.textContent = formatCount(items.length);
  emptyEl.hidden = items.length > 0;
  listEl.replaceChildren(...items.map(renderItem));
}

/** 创建可双击编辑的文本节点；编辑取消/恢复后必须重新绑定，保证能再次编辑 */
function createTextSpan(item) {
  const text = document.createElement("span");
  text.className = "todo-text";
  text.textContent = item.text;
  text.addEventListener("dblclick", () => startEdit(text, item));
  return text;
}

/** 构建单个条目 DOM：勾选框（完成态划线置灰）+ 文本 + 悬停删除按钮 */
function renderItem(item) {
  const li = document.createElement("li");
  li.className = "todo-item" + (item.done ? " done" : "");
  li.dataset.id = item.id;

  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.checked = item.done;
  checkbox.setAttribute("aria-label", item.done ? `取消完成：${item.text}` : `标记完成：${item.text}`);
  checkbox.addEventListener("change", () => onToggle(item.id, checkbox));

  const text = createTextSpan(item);

  const del = document.createElement("button");
  del.type = "button";
  del.className = "delete-btn";
  del.textContent = "\u00d7"; // 关闭符号
  del.setAttribute("aria-label", "删除");
  del.addEventListener("click", () => onRemove(item.id));

  li.append(checkbox, text, del);
  return li;
}

/** 进入编辑态：文本替换为可编辑内容，聚焦并全选；Enter 保存、Esc/失焦取消 */
function startEdit(text, item) {
  const li = text.parentElement;
  if (!li) return;

  // 使用 contenteditable 而不是原生 input，彻底避免 WebView 的原生蓝色下划线
  const editor = document.createElement("span");
  editor.className = "todo-edit-input";
  editor.setAttribute("contenteditable", "true");
  editor.setAttribute("spellcheck", "false");
  editor.setAttribute("autocorrect", "off");
  editor.setAttribute("autocapitalize", "off");
  editor.setAttribute("data-gramm", "false");
  editor.setAttribute("aria-label", "编辑待办");
  editor.textContent = item.text;
  editor.style.cssText = [
    "display:block",
    "box-sizing:border-box",
    "width:100%",
    "min-width:0",
    "border:none",
    "border-bottom:none",
    "outline:none",
    "border-radius:6px",
    "padding:4px 8px",
    "background:rgba(59,130,246,0.08)",
    "color:#1f2937",
    "box-shadow:none",
    "font:inherit",
    "font-size:14px",
    "line-height:1.5",
    "white-space:pre-wrap",
    "word-break:break-word",
    "user-select:text",
    "cursor:text",
  ].join(";");

  text.replaceWith(editor);
  li.classList.add("editing");

  let submitted = false;

  const restore = () => {
    if (!editor.isConnected) return;
    // 用统一创建函数重建文本节点，重新绑定双击编辑，避免失焦取消后无法再次编辑
    const span = createTextSpan(item);
    editor.replaceWith(span);
    li.classList.remove("editing");
  };

  const submit = () => {
    const value = editor.textContent ?? "";
    if (!isValidInput(value) || value.trim() === item.text) {
      submitted = true;
      restore();
      return;
    }
    submitted = true;
    invoke("update_todo", { id: item.id, text: value })
      .then(() => refresh())
      .catch((err) => {
        showBanner(String(err));
        restore();
      });
  };

  editor.addEventListener("focusout", () => {
    if (!submitted) restore();
  });
  editor.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      submit();
    } else if (event.key === "Escape") {
      event.preventDefault();
      submitted = true;
      restore();
    }
  });

  editor.focus();
  const range = document.createRange();
  range.selectNodeContents(editor);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);
}

/** 从后端重新拉取列表并渲染；数据量小，全量刷新保证顺序与状态一致 */
function refresh() {
  return invoke("list_todos").then(render);
}

/** 添加：前端先行校验，空/纯空白不发起调用；成功后清空输入框 */
async function onAdd() {
  const text = inputEl.value;
  if (!isValidInput(text)) {
    return;
  }
  try {
    await invoke("add_todo", { text });
    inputEl.value = "";
    await refresh();
  } catch (err) {
    showBanner(String(err));
  }
}

/** 勾选切换：失败时回滚勾选框状态并提示，界面保持变更前状态 */
async function onToggle(id, checkbox) {
  try {
    await invoke("toggle_todo", { id });
  } catch (err) {
    checkbox.checked = !checkbox.checked;
    showBanner(String(err));
    return;
  }
  await refresh();
}

/** 删除：直接删除无确认；失败时界面保持变更前状态 */
async function onRemove(id) {
  try {
    await invoke("remove_todo", { id });
  } catch (err) {
    showBanner(String(err));
    return;
  }
  await refresh();
}

// 窗口显示后聚焦输入框（后端在 show 后通知）
tauriEvent.listen("card-shown", () => { inputEl.focus(); });
// 窗口隐藏时丢弃未提交输入并清除错误横幅（后端在 hide 时通知）
tauriEvent.listen("card-hidden", () => {
  inputEl.value = "";
  bannerEl.hidden = true;
});

// 输入框回车提交
inputEl.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    onAdd();
  }
});

// 全局关闭 WebKit 拼写/自动更正/自动大写的提示，避免编辑区出现蓝色下划线
for (const el of [document.documentElement, document.body]) {
  el.setAttribute("spellcheck", "false");
  el.setAttribute("autocorrect", "off");
  el.setAttribute("autocapitalize", "off");
}
try {
  document.execCommand("enableInlineSpellChecking", false, false);
  document.execCommand("enableGrammarChecking", false, false);
} catch (err) {
  // 部分 WebView 不支持这两个命令，忽略即可
}

// 启动加载既有列表（重启恢复）
refresh().catch((err) => showBanner(String(err)));
