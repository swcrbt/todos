// 前端纯逻辑：浏览器与 Node 双端可加载的 ES Module（零构建）。
// 契约与原型 assets/logic.mjs 保持一致（真实交互路径直接复用本文件）。

/**
 * 输入有效性判断（空/纯空白不产生条目，前端侧）。
 * - 非字符串（null/undefined/数字/对象）→ false（防御分支）
 * - 字符串 trim 后长度为 0 → false（空格/Tab/换行均视为空输入）
 * @param {*} text 待校验文本
 * @returns {boolean} trim 后非空为 true
 */
export function isValidInput(text) {
  return typeof text === "string" && text.trim().length > 0;
}

/**
 * 计数徽标中文文案。
 * 防御：非数字或负数一律返回 "共 0 项"。
 * @param {*} count 条目数
 * @returns {string} 如 "共 3 项"
 */
export function formatCount(count) {
  if (typeof count !== "number" || !Number.isFinite(count) || count < 0) {
    return "共 0 项";
  }
  return `共 ${count} 项`;
}