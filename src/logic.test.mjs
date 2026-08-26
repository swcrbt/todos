// 前端纯逻辑单测（`node --test src/logic.test.mjs`，Node 内置 test runner，零依赖）。
// 契约来源：isValidInput / formatCount 双端 ES Module。

import { test } from "node:test";
import assert from "node:assert/strict";
import { isValidInput, formatCount } from "./logic.mjs";

// ===== isValidInput：空/纯空白拒绝 =====

test("isValidInput: 空字符串返回 false", () => {
  assert.equal(isValidInput(""), false);
});

test("isValidInput: 纯空白（空格/Tab/换行）返回 false", () => {
  assert.equal(isValidInput("   "), false);
  assert.equal(isValidInput("\t"), false);
  assert.equal(isValidInput("\n"), false);
  assert.equal(isValidInput(" \t \n "), false);
});

test("isValidInput: 非字符串输入返回 false（防御分支）", () => {
  assert.equal(isValidInput(null), false);
  assert.equal(isValidInput(undefined), false);
  assert.equal(isValidInput(42), false);
  assert.equal(isValidInput({}), false);
});

test("isValidInput: 中文文本返回 true", () => {
  assert.equal(isValidInput("买牛奶"), true);
});

test("isValidInput: 首尾空格的中文文本返回 true（trim 后校验）", () => {
  assert.equal(isValidInput("  开会  "), true);
});

// ===== formatCount：中文计数文案与防御 =====

test("formatCount: 0/1/3 输出中文文案", () => {
  assert.equal(formatCount(0), "共 0 项");
  assert.equal(formatCount(1), "共 1 项");
  assert.equal(formatCount(3), "共 3 项");
});

test("formatCount: 非数字返回 '共 0 项'（防御要求）", () => {
  assert.equal(formatCount("3"), "共 0 项");
  assert.equal(formatCount(undefined), "共 0 项");
  assert.equal(formatCount(null), "共 0 项");
  assert.equal(formatCount(NaN), "共 0 项");
});

test("formatCount: 负数返回 '共 0 项'（防御要求）", () => {
  assert.equal(formatCount(-1), "共 0 项");
});