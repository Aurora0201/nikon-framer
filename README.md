# Nikon Framer 📷

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?style=flat&logo=tauri&logoColor=black)
![Rust](https://img.shields.io/badge/Rust-1.75+-000000?style=flat&logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows-lightgrey)
![License](https://img.shields.io/badge/License-MIT-green)

**Nikon Framer** 是一个专为尼康（Nikon）摄影师打造的高性能、隐私安全的本地水印相框生成工具。

基于 **Rust** 和 **Tauri v2** 构建，它结合了系统级原生应用的极致性能与轻量级 Web 前端的灵活性。无需上传图片到云端，利用本地算力毫秒级完成合成，完美支持 4000万+ 像素 RAW 转 JPG 大图处理。

---

## ✨ 核心特性 (Features)

### 🚀 强劲内核
* **极速处理**: 依托 Rust 的 `image-rs` 库与多线程并发模型，处理高分辨率照片流畅无卡顿。
* **智能 EXIF 识别**: 自动解析相机型号（如 Nikon Z8）、镜头参数（如 50mm f/1.8）、ISO、快门及光圈数据。
* **隐私优先**: 纯本地运行，断网可用，您的照片数据永远不会离开您的硬盘。

### 🛠️ 强大的批处理 (New!)
* **队列管理**: 支持拖拽文件或文件夹快速导入，列表支持一键清空或单项移除。
* **可视化进度**: 底部实时进度条显示处理百分比与当前文件名，直观掌握任务状态。
* **安全控制**: 内置防误触机制，支持处理过程中随时**安全终止**任务。
* **智能过滤**: 自动剔除不支持的文件格式或无 EXIF 信息的图片，确保流程不中断。

### 🎨 影廊级美学
* **高斯模糊 (Atmosphere)**: 智能提取画面主色调生成模糊背景，辅以磨砂玻璃质感与弥散投影，营造高端氛围。
* **极简白底 (Gallery)**: 经典的画廊风格白边，仿拍立得布局，专注于影像本身的纯粹。
* **完美排版**:
    * 专为 Nikon Z 系列优化的 Logo 布局（Z Logo + 品牌字）。
    * **胶囊式序号设计**: 精致的列表索引样式，细节处见真章。
    * 动态字体渲染，支持斜体型号显示与像素级对齐修正。

---

## 📸 效果预览 (Preview)

| **高斯模糊风格 (Blur)** | **极简白底风格 (White)** |
| :---: | :---: |
| ![blur](imgs/blur.jpg) | ![white](imgs/white.jpg) |

*(此处建议放一张软件主界面的截图，展示新的进度条和列表 UI)*

---

## 🛠️ 技术栈 (Tech Stack)

本项目采用现代化的 **Rust + Tauri** 混合架构：

* **Frontend**: HTML5 / CSS3 / Vanilla JS (无框架依赖，极致轻量，启动秒开)
* **Backend**: Rust (核心业务逻辑)
* **关键库 (Crates)**:
    * `tauri`: 跨平台应用框架 (v2)
    * `image`: 高性能并行图像编解码
    * `kamadak-exif`: 专业级元数据解析
    * `ab_glyph`: 矢量字体加载与排版引擎
    * `serde`: 高效的数据序列化与前后端通信

---

## 🚀 开发指南 (Development)

如果您想在本地运行或贡献代码，请确保已安装 Rust 和 Node.js 环境。

### 1. 克隆项目

```bash
git clone [https://github.com/Aurora0201/nikon-framer.git](https://github.com/Aurora0201/nikon-framer.git)
cd nikon-framer
```

### 2. 安装依赖
```Bash
npm install
```
### 3. 开发模式运行
启动前端与 Rust 后端的热重载开发环境：

```Bash

npm run tauri dev
```
⚠️ 性能提示: 在 dev 模式下，Rust 编译器未开启优化，大图处理速度可能会较慢。如需体验真实性能，请使用 release 模式构建。


### 4. 打包构建
构建 Windows 安装包 (.msi / .exe) 或 macOS 应用 (.dmg / .app)：

```Bash

npm run tauri build
```
构建产物将位于 src-tauri/target/release/bundle 目录下。

### 📝 待办事项 (Todo)
- [x] 批量处理模式 (Batch Processing) ✅ 已完成

- [ ] 支持更多品牌 Logo (Sony, Canon, Fujifilm)

- [ ] 自定义水印签名 (User Signature)

- [ ] 更多样式的边框模板 (黑底、胶片风等)

- [ ] 导出路径自定义

📄 许可证 (License)
MIT License © 2025 Aurora0201
