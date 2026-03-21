# 安装指南

安装方式分为两种：

- 从 Release 下载
    - 适合快速体验、测试和本地使用
    - 不需要本地构建前后端
- 从源码编译
    - 适合开发、调试和参与项目修改
    - 需要安装 Rust 和前端工具链

普通用户建议选择**从 Release 下载**

## 方式一：从 Release 下载

### 1. 下载发布包

- 打开 GitHub Releases
- 根据你的平台下载对应压缩包
- 解压到本地目录，并保持目录结构不变

可用产物示例：

- Linux: `sillystage-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `sillystage-x86_64-pc-windows-msvc.zip`

### 2. 了解解压后的目录

发布包通常包含：

- 可执行文件
- `ss-app.toml`
- `webapp/`
- `data/`（首次运行时可自动创建）

说明：

- `ss-app.toml` 是默认配置文件
- `webapp/` 是已经构建好的前端静态资源
- `data/` 用于本地存储数据

### 3. 启动应用

Linux:

```bash
./ss-app
```

Windows:

```powershell
.\ss-app.exe
```

或者在文件浏览器中双击`ss-app.exe`启动

## 方式二：从源码编译

### 1. 前置要求

- Rust toolchain
- Cargo
- pnpm
- just（推荐）

### 2. 拉取源码

```bash
git clone <repo-url>
cd SillyStage
```

### 3. 安装前端依赖

应用前端：

```bash
cd webapp
pnpm install
cd ..
```

### 4. 开发模式启动

推荐方式：

```bash
just dev
```

这会同时启动：

- Rust 后端
- webapp 前端开发服务器

如果只想启动后端：

```bash
just backend
```
