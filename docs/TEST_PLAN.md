# Ring 项目 — 完整测试计划

## 测试环境准备

### 1. 启动后端
```bash
cd /Users/kaiiangs/Desktop/open-source-project/Ring/server
cargo run
```

### 2. 启动前端（开发模式）
```bash
cd /Users/kaiiangs/Desktop/open-source-project/Ring/ui
npm run dev
```

### 3. 访问地址
- 前端: http://localhost:5173
- 后端: http://localhost:7420

---

## 测试用例清单

### 第一阶段：Setup流程测试

| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 1 | 首次访问 | 打开 http://localhost:5173 | 显示Setup向导，Step 1 Welcome |
| 2 | 身份设置 | 输入Display Name, 选择Avatar | 进入Step 2 Identity |
| 3 | LLM配置 | 选择Provider, 输入Model, API Key | TEST CONNECTION按钮可用 |
| 4 | LLM测试 | 点击TEST CONNECTION | 显示成功/失败状态 |
| 5 | GitLab配置 | 输入GitLab URL和Token（或Skip） | 可以跳过GitLab配置 |
| 6 | 完成Setup | 点击Done | 显示命令速查表，进入主界面 |
| 7 | Token恢复 | 刷新页面 | 自动恢复登录状态 |

### 第二阶段：核心功能测试

#### 2.1 Ring管理
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 8 | 创建Ring | 侧边栏点击+，输入名称和描述 | 新Ring出现在列表中 |
| 9 | 切换Ring | 点击不同Ring | 聊天区域显示对应Ring的聊天 |
| 10 | Ring配置 | 点击Config面板 | 显示成员、邀请、导出功能 |

#### 2.2 聊天系统
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 11 | Group Ring聊天 | 输入消息，点击发送 | AI回复，消息显示token用量 |
| 12 | Super Ring聊天 | 切换到Super Ring | 显示全局助手，有工具调用能力 |
| 13 | Self聊天 | 点击右下角猫图标或输入@self | 打开Self浮窗，可对话 |
| 14 | @self转发 | 在Group Ring输入@self 测试消息 | 消息转发到Self，Self浮窗打开 |
| 15 | 命令补全 | 输入/或@ | 显示命令补全弹出框 |
| 16 | 命令执行 | 输入/graph | 打开图谱面板，不发送给AI |

#### 2.3 图谱系统
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 17 | 查看图谱 | 输入/graph | 显示力导向图，有节点和边 |
| 18 | 创建节点 | 输入节点标签，点击+Node | 新节点出现在图谱中 |
| 19 | 删除节点 | 输入"删掉xxx节点"（Graph对话修正） | AI执行删除操作 |
| 20 | 标签过滤 | 在GraphPanel选择标签 | 只显示对应标签的节点 |
| 21 | 展开/折叠 | 点击有子节点的+/- | 子节点显示/隐藏 |
| 22 | 导出图谱 | 点击Export按钮 | 下载graph.json文件 |

#### 2.4 归档系统
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 23 | 创建归档 | 输入/save | 显示归档进度，完成后提交 |
| 24 | 查看归档 | 打开Archive面板 | 显示归档列表 |
| 25 | PR Review | 点击待审归档 | 显示diff内容，可merge/reject |
| 26 | Diff视图 | 点击View Diff | 显示代码对比 |

#### 2.5 Session系统
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 27 | 创建Session | 输入/session create | 创建新Session |
| 28 | 材料准备 | 创建非discussion Skill的Session | 自动显示生成的材料 |
| 29 | 开始讨论 | 点击Start | 进入discussion阶段 |
| 30 | 实时聊天 | 在Session中发送消息 | 其他参与者实时收到 |
| 31 | 导出Session | 点击Export | 下载session.md文件 |

#### 2.6 Self系统
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 32 | Self Memory | 打开Self浮窗，点击Memory标签 | 显示聊天统计、Session统计 |
| 33 | Personality | 打开Settings，切换Tone | 保存后AI语气改变 |
| 34 | Privacy设置 | 打开Privacy，调整级别 | 设置保存成功 |
| 35 | 数据导出 | 点击EXPORT | 下载self-data.json |
| 36 | 数据重置 | 点击RESET，确认 | 所有数据清空 |

#### 2.7 通知系统
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 37 | 通知铃铛 | 点击右上角铃铛 | 显示通知列表 |
| 38 | 标记已读 | 点击通知 | 标记为已读，未读数减少 |

#### 2.8 导出中心
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 39 | 聊天导出 | 点击Export按钮 | 下载chat.md文件 |
| 40 | 备份导出 | 在Config面板点击Full Ring Backup | 下载backup.json |

#### 2.9 Blueprint系统
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 41 | 选择模板 | 输入/blueprint | 打开Blueprint面板 |
| 42 | 应用模板 | 选择模板，预览，确认 | Ring结构按模板创建 |

#### 2.10 Context管理
| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 43 | 自动Compact | 发送超过30条消息 | 旧消息被压缩为摘要 |
| 44 | Ephemeral模式 | （需前端支持）发送ephemeral消息 | 消息不保存到历史 |

---

### 第三阶段：AI自动化测试

| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 45 | .group维护 | 在Ring中聊天超过3轮 | 检查~/.ring/rings/{id}/.group/active-context是否更新 |
| 46 | Archive模式 | 成功归档几次后 | 检查~/.ring/rings/{id}/.group/archive-patterns是否更新 |
| 47 | 知识摘要 | 创建/删除图谱节点 | 检查~/.ring/rings/{id}/.group/knowledge-summary是否更新 |

---

### 第四阶段：边界测试

| # | 测试项 | 操作步骤 | 预期结果 |
|---|--------|----------|----------|
| 48 | 空消息 | 发送空消息 | 提示或忽略 |
| 49 | 超长消息 | 粘贴10000字文本 | 正常发送，显示完整 |
| 50 | 并发聊天 | 快速发送多条消息 | 消息顺序正确，不丢失 |
| 51 | 离线恢复 | 关闭浏览器，重新打开 | 自动恢复token，回到之前状态 |

---

## 测试检查表

- [ ] 所有16个功能都已实现
- [ ] 55个后端测试通过
- [ ] 前端构建成功
- [ ] 手动测试所有核心流程
- [ ] 边界条件测试通过
- [ ] UI/UX无明显问题

---

## 截图检测点

使用Playwright截图以下关键页面：

1. **Setup向导** - 5个步骤的截图
2. **主界面** - 三栏布局
3. **聊天界面** - 显示AI回复和token用量
4. **图谱面板** - 显示节点和边
5. **Archive面板** - 显示归档列表和diff
6. **Session面板** - 显示材料准备和实时聊天
7. **Self浮窗** - Memory/Settings标签页
8. **Blueprint面板** - 模板选择
9. **命令补全** - 显示补全弹出框
10. **通知铃铛** - 显示通知列表

---

## 测试脚本

```python
# test_ring.py
from playwright.sync_api import sync_playwright
import time

def test_setup():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=False)
        page = browser.new_page(viewport={'width': 1440, 'height': 900})
        
        # 访问首页
        page.goto('http://localhost:5173')
        page.screenshot(path='screenshots/01-setup-welcome.png')
        
        # 完成Setup...
        
        browser.close()

if __name__ == '__main__':
    test_setup()
```

---

## 预期测试时间

- 第一阶段（Setup）: 5分钟
- 第二阶段（核心功能）: 30分钟
- 第三阶段（AI自动化）: 10分钟
- 第四阶段（边界测试）: 10分钟
- 截图检测: 10分钟
- **总计: 约65分钟**

---

## 反馈模板

测试完成后，请反馈：

1. **发现的Bug**: [描述]
2. **UI/UX问题**: [描述]
3. **性能问题**: [描述]
4. **功能建议**: [描述]
5. **整体评分**: 1-10分
