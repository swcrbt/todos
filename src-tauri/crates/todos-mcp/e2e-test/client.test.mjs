// 真实 MCP 客户端端到端测试：通过官方 SDK 的 StdioClientTransport 连接 todos-mcp，
// 验证 initialize → tools/list → tools/call 全流程与桌面应用共享数据文件。
// 运行方法（需先 cargo build -p todos-mcp）：
//   cd src-tauri/crates/todos-mcp/e2e-test
//   npm install @modelcontextprotocol/sdk   # 或 ln -s 指向已有 node_modules
//   node client.test.mjs
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import { mkdirSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";

const serverBin = new URL("../../../target/debug/todos-mcp", import.meta.url).pathname;
const dataDir = new URL("./.e2e-data/", import.meta.url).pathname;
mkdirSync(dataDir, { recursive: true });
const dataPath = join(dataDir, "todos.json");

const transport = new StdioClientTransport({
  command: serverBin,
  args: [],
  env: { ...process.env, TODOS_DATA_PATH: dataPath },
  stderr: "inherit",
});
const client = new Client({ name: "e2e-test", version: "0.0.1" });

let failures = 0;
function check(name, cond, detail = "") {
  console.log(`${cond ? "✓" : "✗"} ${name}${cond ? "" : " — " + detail}`);
  if (!cond) failures++;
}

try {
  await client.connect(transport);
  check("initialize 握手", client.getServerVersion()?.name === "todos-mcp");

  const { tools } = await client.listTools();
  const names = tools.map((t) => t.name).sort();
  check(
    "tools/list 返回五个工具",
    JSON.stringify(names) ===
      JSON.stringify(["add_todo", "list_todos", "remove_todo", "toggle_todo", "update_todo"]),
    names.join(","),
  );

  const added = await client.callTool({ name: "add_todo", arguments: { text: "  写端到端测试  " } });
  const addedItem = JSON.parse(added.content[0].text);
  check("add_todo 返回条目并 trim", addedItem.text === "写端到端测试" && !!addedItem.id);

  const toggled = await client.callTool({ name: "toggle_todo", arguments: { id: addedItem.id } });
  check("toggle_todo 置为完成", JSON.parse(toggled.content[0].text).done === true);

  const updated = await client.callTool({
    name: "update_todo",
    arguments: { id: addedItem.id, text: "写真实客户端端到端测试" },
  });
  check("update_todo 修改文本", JSON.parse(updated.content[0].text).text === "写真实客户端端到端测试");

  const listed = await client.callTool({ name: "list_todos", arguments: {} });
  const items = JSON.parse(listed.content[0].text);
  check("list_todos 返回 1 条", items.length === 1 && items[0].done === true);

  const blank = await client.callTool({ name: "add_todo", arguments: { text: "   " } });
  check("空白文本 isError", blank.isError === true);

  const notFound = await client.callTool({ name: "toggle_todo", arguments: { id: "不存在" } });
  check("未知 id isError", notFound.isError === true);

  const removed = await client.callTool({ name: "remove_todo", arguments: { id: addedItem.id } });
  check("remove_todo 删除", removed.isError !== true);

  // 落盘格式与桌面应用契约一致（version/items），删除后为空
  const file = JSON.parse(readFileSync(dataPath, "utf8"));
  check("数据文件契约 version=1 且为空库", file.version === 1 && file.items.length === 0);
} finally {
  await client.close();
  rmSync(dataDir, { recursive: true, force: true });
}

console.log(failures === 0 ? "\n全部通过" : `\n${failures} 项失败`);
process.exit(failures === 0 ? 0 : 1);
