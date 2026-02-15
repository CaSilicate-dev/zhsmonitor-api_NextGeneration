# ZHSMonitor / CaSilicate 服务器监控站

ZHSMonitor 的全称是 [ZHServer](https://www.fts427.top/Minecraft/ZHS/) Monitor，最初是为
[FTS427](https://github.com/FTS427) 的 Minecraft 服务器(ZHServer)开发的监控工具。
后来加入了其他服务器的监控，因此项目有两个名字：GitHub 仓库名(正式项目名)
使用 ZHSMonitor， 也可以称为 CaSilicate 服务器监控站。

## 监控对象

1. ZHS MC 服务器（FTS427 的 Minecraft 服务器）
2. systemdirect 的 MC 服务器（无正式名称）
3. TRC MC 服务器（红石之都 MC 服务器）
4. 作者的 VPS 服务器（系统监控） 
5. 作者的物理机服务器（系统监控）

## 数据压缩

由于前端无法展示如此多的数据,我们固定了前端只展示 300 个左右的数据

可选时间范围：300秒（精确数据）、1小时、1天、10天、100天、1年。  
采用时间分桶聚合算法，在长周期查询时保留数据特征。

## 技术栈

- 前端：React + Ant Design
- 后端：Rust
- 数据库：MySQL

## 说明

这个项目是我个人和我朋友使用的专用监控工具，并非通用解决方案。所以我们不提供安装方法，因为在不修改代码的情况下没有任何其他作用