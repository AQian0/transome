# Transome

> 🚧 **注意**: 该项目目前处于开发阶段，尚未达到生产级别！

一个简洁高效的命令行AI翻译工具，专为终端工作流设计。

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

## ✨ 特性

- 🚀 **高性能**: 基于 Rust 构建，启动快速，资源占用低
- 🤖 **多模型支持**: 支持 Google Gemini 和 OpenAI GPT 系列模型
- 🔄 **智能翻译**: 自动识别中英文，双向翻译
- ⚙️ **灵活配置**: 支持自定义API端点、模型和翻译提示词
- 🎯 **简洁输出**: 专注翻译结果，无多余信息
- 🔧 **终端友好**: 专为命令行工作流优化

## 🚀 快速开始

### 安装

目前支持从源码构建：

```bash
# 克隆仓库
git clone https://github.com/your-username/transome.git
cd transome

# 构建项目
cargo build --release

# 安装到系统路径（可选）
cargo install --path .
```

### 配置

设置你的 API 密钥：

```bash
# 使用 Google Gemini（推荐）
export GOOGLE_AI_API_KEY="your-api-key-here"

# 或者使用 OpenAI
export OPENAI_API_KEY="your-api-key-here"
```

## 📖 使用方法

### 基本用法

```bash
# 英译中
transome "Hello world"
# 输出: 你好，世界

# 中译英
transome "你好世界"
# 输出: hello world
```

### 高级用法

```bash
# 指定模型
transome -m gpt-4 "Hello world"

# 使用自定义API端点
transome -u https://custom.api.com/v1 -m custom-model "Hello world"

# 自定义翻译提示词
transome -p "请翻译成正式的商务英语" "你好"

# 查看所有支持的模型
transome --list-models
```

## 🔧 命令行选项

| 选项 | 简写 | 描述 | 默认值 |
|------|------|------|--------|
| `--model` | `-m` | 指定AI模型 | `gemini-2.5-flash-lite` |
| `--url` | `-u` | 自定义API端点URL | 根据模型自动选择 |
| `--key` | `-k` | API密钥 | 从环境变量读取 |
| `--prompt` | `-p` | 自定义翻译提示词 | 内置智能提示词 |
| `--list-models` | | 列出所有支持的模型 | |
| `--help` | `-h` | 显示帮助信息 | |
| `--version` | `-V` | 显示版本信息 | |

## 🤖 支持的模型

### Google Gemini
- `gemini-1.5-flash`
- `gemini-1.5-pro`
- `gemini-2.5-flash` 
- `gemini-2.5-flash-lite` (默认)
- `gemini-2.5-pro`

### OpenAI GPT
- `gpt-3.5-turbo`
- `gpt-3.5-turbo-16k`
- `gpt-4`
- `gpt-4-turbo`
- `gpt-4o`
- `gpt-4o-mini`

## 💡 设计理念

**为什么选择 Transome？**

- 🎯 **专注**: 专门为翻译任务优化，无多余功能
- ⚡ **高效**: 终端原生，无需切换窗口或打开浏览器
- 🔒 **隐私**: 支持自定义API端点，数据处理透明
- 🛠️ **灵活**: 支持多种AI模型和自定义配置

## 🗺️ 开发路线图

### 🔥 高优先级

- [ ] 完善请求与响应类型定义
- [ ] 添加更多AI模型支持
- [ ] 实现配置文件支持
- [ ] 添加批量翻译功能

### 📅 计划中

- [ ] 多语言翻译支持（不限于中英互译）
- [ ] 翻译历史记录
- [ ] 输出格式美化
- [ ] Shell 集成和补全
- [ ] 二进制分发支持

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](./LICENSE) 文件了解详情。
