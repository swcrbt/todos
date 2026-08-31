// 前端纯逻辑（原型内契约演示；真实实现路径为 Build 的 src/logic.mjs）
// 与 design.md INT-005 对应：无副作用、浏览器与 Node 双端可加载。

/**
 * 输入有效性判断（REQ-TODO-001）：空/纯空白文本不产生条目。
 * @param {string|null|undefined} text
 * @returns {boolean} trim 后非空为 true
 */
export function isValidInput(text) {
  return typeof text === "string" && text.trim().length > 0;
}

/**
 * 计数徽标文案（中文，FACT-009）。
 * @param {*} count
 * @returns {string} 如 "共 3 项"
 */
export function formatCount(count) {
  if (typeof count !== "number" || !Number.isFinite(count) || count < 0) {
    return "共 0 项";
  }
  return `共 ${count} 项`;
}