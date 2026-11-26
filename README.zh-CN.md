# OrbitSwitch · Antigravity 桌面版

[English](README.md) · [从源码构建](docs/release.md) · [兼容与验证记录](docs/status.md)

一个轻量菜单栏工具，用来手动切换 Antigravity 桌面端的 Google 账号，并查看各账号的套餐和剩余额度。

<picture><source media="(prefers-color-scheme: dark)" srcset="assets/screenshots/hero-zh-dark.png"><img src="assets/screenshots/hero-zh-light.png" width="460" alt="OrbitSwitch 界面，使用虚构的 Free 和 Pro 账号"></picture>

- **切换账号。** 自动识别当前登录。在浏览器授权第二个账号后，点账号卡片即可切换并重新打开桌面端。
- **查看额度。** 显示 Google 返回的剩余额度和重置时间。Free 账号只显示套餐和无额度明细的提示。
- **设置网络。** 从 **··· → 网络设置** 选择跟随系统网络或使用本机 HTTP 代理，保存前可以测试连接。

**平台：** macOS 26+。Windows 10/11 x64 可从源码构建，尚未在 Windows 实机验证换号。Antigravity 各版本的验证范围见[验证记录](docs/status.md)。

[MIT](LICENSE) · 基于 [OrbitSwitch](https://github.com/shengyy/orbitswitch) 的桌面专用 fork。
