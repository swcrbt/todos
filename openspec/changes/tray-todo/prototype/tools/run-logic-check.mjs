// 快速契约检查：原型 logic.mjs（INT-005 / SEAM-002 契约的预验证）
// 运行：node tools/run-logic-check.mjs（在 prototype/ 目录下）
import { strict as assert } from "node:assert";
import { isValidInput, formatCount } from "../assets/logic.mjs";

// SCN-TODO-001-B：空/纯空白不添加
assert.equal(isValidInput(""), false);
assert.equal(isValidInput("   "), false);
assert.equal(isValidInput("\t\n "), false);
// SCN-TODO-001-A：非空（含 trim 后非空）可添加
assert.equal(isValidInput("买牛奶"), true);
assert.equal(isValidInput("  买牛奶  "), true);
// 防御：非字符串
assert.equal(isValidInput(null), false);
assert.equal(isValidInput(undefined), false);
// FACT-009 中文文案
assert.equal(formatCount(3), "共 3 项");
assert.equal(formatCount(0), "共 0 项");
console.log("LOGIC_CONTRACT_OK");