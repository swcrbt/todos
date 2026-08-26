// 原型 mock 数据层：模拟 design.md MOD-001（todo-store）的公开行为。
// 所有 mock 数据集中于此文件，仅存在于 prototype/ 目录；真实运行路径由
// Build 依据 design.md 实现为 Rust todo-store crate + todos.json。
// 持久化使用 localStorage 模拟 `app_data_dir/todos.json`；数据格式与真实
// 文件保持一致：{ version: 1, items: [{ id, text, done, created_at }] }。

import { isValidInput } from "./logic.mjs";

const STORAGE_KEY = "trayTodo.prototype.v1";
const SCHEMA_VERSION = 1;

/** 原型唯一存储源；save 失败时（模拟写盘失败）不提交变更。 */
let state = load();

/** 模拟写盘失败开关（原型受限状态演示，对应真实 command 的持久化错误）。 */
let persistFail = false;

const listeners = new Set();

function load() {
  try {
    const raw = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && parsed.version === SCHEMA_VERSION && Array.isArray(parsed.items)) {
        return { version: SCHEMA_VERSION, items: parsed.items };
      }
    }
  } catch (err) {
    // 损坏数据：原型内丢弃并重新开始（真实实现为备份损坏文件 + 空库启动）
    console.error("[mock] 本地数据损坏，重置为空库", err);
  }
  return { version: SCHEMA_VERSION, items: [] };
}

function save(nextState) {
  if (persistFail) {
    throw new Error("模拟保存失败：写入磁盘出错");
  }
  try {
    globalThis.localStorage.setItem(STORAGE_KEY, JSON.stringify(nextState));
  } catch (err) {
    throw new Error(`写入磁盘出错：${err.message}`);
  }
}

function commit(nextState) {
  save(nextState);   // 先持久化，成功后切换内存状态（INV-002 失败不改内存）
  state = nextState;
  for (const fn of listeners) fn();
}

export function subscribe(fn) {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

/** @returns {Array<{id:string,text:string,done:boolean,created_at:number}>} */
export function list() {
  return state.items.map((item) => ({ ...item }));
}

export function add(text) {
  if (!isValidInput(text)) {
    throw new Error("输入为空，不添加待办");
  }
  const item = {
    id: crypto.randomUUID(),
    text: text.trim(),
    done: false,
    created_at: Date.now(),
  };
  // 追加到队尾（INV-004：顺序 = 添加顺序）
  commit({ version: SCHEMA_VERSION, items: [...state.items, item] });
  return item;
}

export function toggle(id) {
  const target = state.items.find((item) => item.id === id);
  if (!target) {
    throw new Error(`待办不存在：${id}`);
  }
  const items = state.items.map((item) =>
    item.id === id ? { ...item, done: !item.done } : item
  );
  commit({ version: SCHEMA_VERSION, items });
  return items.find((item) => item.id === id);
}

export function remove(id) {
  const exists = state.items.some((item) => item.id === id);
  if (!exists) {
    throw new Error(`待办不存在：${id}`);
  }
  commit({ version: SCHEMA_VERSION, items: state.items.filter((item) => item.id !== id) });
}

export function setPersistFail(value) {
  persistFail = Boolean(value);
}

export function getPersistFail() {
  return persistFail;
}

export function reset() {
  try {
    globalThis.localStorage.removeItem(STORAGE_KEY);
  } catch (err) {
    console.error("[mock] 清除本地数据失败", err);
  }
  state = { version: SCHEMA_VERSION, items: [] };
  for (const fn of listeners) fn();
}

// 原型调试钩子（仅 prototype 使用；CDP 验证与手动调试用，不属于真实运行路径）
export const __protoDebug = {
  getItems: () => JSON.parse(JSON.stringify(state.items)),
  storageKey: STORAGE_KEY,
};