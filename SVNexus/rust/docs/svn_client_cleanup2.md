# 03/08 00:53 SVN清理函数解析

> 解析libsvn中的svn_client_cleanup2函数



**svn_client_cleanup2 函数解析（libsvn_client 库）**

该函数位于 Subversion（SVN）客户端库 `libsvn_client` 中，是 Subversion 1.9 引入的**工作副本清理 API**（对应命令行 `svn cleanup` 的核心实现）。它用于修复因操作中断导致的工作副本不一致状态（例如“working copy locked”错误）。

### 函数签名
```c
svn_error_t *
svn_client_cleanup2(const char *dir_abspath,
                    svn_boolean_t break_locks,
                    svn_boolean_t fix_recorded_timestamps,
                    svn_boolean_t clear_dav_cache,
                    svn_boolean_t vacuum_pristines,
                    svn_boolean_t include_externals,
                    svn_client_ctx_t *ctx,
                    apr_pool_t *scratch_pool);
```

**返回值**：`svn_error_t *`（成功返回 `SVN_NO_ERROR`，失败返回错误对象）。

**引入版本**：New in Subversion 1.9（旧版兼容函数 `svn_client_cleanup` 已标记 deprecated）。

### 功能描述
递归清理指定工作副本目录 `dir_abspath`：
- 完成任何未完成的操作（unfinished operations，如工作队列 work_queue 中的任务）。
- 移除工作副本锁文件（.svn/locks 等）。
- 修复管理记录（administrative records）。
- 根据参数执行额外清理任务（时间戳修复、DAV 缓存清除、pristine 存储清理、externals 递归等）。

最终使工作副本恢复到一致、可操作状态。

### 参数详细说明
- **`dir_abspath`**（const char *）：**必须是绝对路径**的工作副本目录（函数内部会断言检查 `svn_dirent_is_absolute`）。支持根目录或子目录。
- **`break_locks`**（svn_boolean_t）：`TRUE` 时强制打破（break）该路径及其子目录的所有现有锁（紧急解锁场景）；`FALSE` 时正常获取写锁。
- **`fix_recorded_timestamps`**（svn_boolean_t）：`TRUE` 时修复未修改文件的记录时间戳，减少后续 `svn status`、`svn diff` 等操作的比较时间。
- **`clear_dav_cache`**（svn_boolean_t）：`TRUE` 时清除旧版 mod_dav 服务器的 DAV 缓存（针对 pre-HTTPv2 仓库，失效部分缓存数据）。
- **`vacuum_pristines`**（svn_boolean_t）：`TRUE` 且 `dir_abspath` 是工作副本根目录时，删除 pristine store（.svn/pristine/）中未引用的文件，释放磁盘空间（对应命令行 `--vacuum-pristines`）。
- **`include_externals`**（svn_boolean_t）：`TRUE` 时递归进入 externals 定义的外部工作副本并一起清理。
- **`ctx`**（svn_client_ctx_t *）：客户端上下文，提供取消回调（`cancel_func`）、通知回调等。如果 `ctx->cancel_func` 非空，会在操作中定期调用，中断时返回 `SVN_ERR_CANCELLED`。
- **`scratch_pool`**（apr_pool_t *）：临时内存池（用于本次操作的临时分配，操作结束后自动释放）。

### 与旧版函数的区别（svn_client_cleanup）
旧版（1.8 及以前兼容）：
```c
svn_error_t * svn_client_cleanup(const char *dir, svn_client_ctx_t *ctx, apr_pool_t *scratch_pool);
```
- 路径不强制绝对。
- **不支持** `break_locks`、`fix_recorded_timestamps`、`clear_dav_cache`、`vacuum_pristines`、`include_externals` 等精细控制。
- 功能受限，仅基本清理。
- 已废弃，仅为兼容保留。**推荐所有新代码使用 `svn_client_cleanup2`**。

### 相关函数
- **`svn_client_vacuum`**：专门的“真空”清理版本（1.9 新增），额外支持 `remove_unversioned_items` / `remove_ignored_items`（对应 `--remove-unversioned` / `--remove-ignored`），但不处理锁/中断操作。
- 底层调用：`libsvn_client/cleanup.c` 中委托给 `libsvn_wc` 库的 `svn_wc_cleanup4`（处理 `.svn/wc.db` 数据库、锁、pristine 存储、工作队列等）。

### 使用场景与注意事项
- **典型场景**：工作副本因网络中断、进程崩溃、kill 操作等出现锁错误时调用。
- GUI 客户端（如 TortoiseSVN）在“Cleanup”对话框中会根据用户勾选选项设置这些 boolean 参数。
- **性能提示**：`vacuum_pristines` 只在根目录有效；`include_externals` 会增加递归开销。
- **取消支持**：通过 `ctx` 可中断长耗时清理。
- **命令行对应**：
  - `svn cleanup` 默认调用简化版。
  - 1.9+ 支持 `--break-locks`、`--vacuum-pristines`、`--include-externals` 等选项。

**官方文档链接**：[Cleanup Group - Subversion API](https://subversion.apache.org/docs/api/latest/group__Cleanup.html)
[完整头文件参考](https://subversion.apache.org/docs/api/latest/svn__client_8h_source.html)

此函数是 libsvn_client 对外暴露的公共接口，内部实现细节（wc 层数据库操作）属于私有实现。如需 C 调用示例代码或更深层的 `libsvn_wc` 解析，可提供更多细节进一步说明。

-----

> 调用svn cleanup命令的时候实际上是往svn_client_cleanup2传入了什么参数



**调用 `svn cleanup` 命令时实际传入 `svn_client_cleanup2` 的参数如下（基于 Subversion trunk/current 源码 `subversion/svn/cleanup-cmd.c`）：**

### 1. 普通情况（默认命令 `svn cleanup [PATH]`，**不带** `--remove-unversioned`、`--remove-ignored` 或 `--vacuum-pristines`）
代码走 `else` 分支，直接调用：

```c
svn_client_cleanup2(target_abspath,
                    TRUE,  /* break_locks */
                    TRUE,  /* fix_recorded_timestamps (fix_timestamps) */
                    TRUE,  /* clear_dav_cache */
                    TRUE,  /* vacuum_pristines */
                    opt_state->include_externals,  /* include_externals */
                    ctx,
                    iterpool);
```

- **`dir_abspath`**：目标路径的**绝对路径**（`target_abspath`，通过 `svn_dirent_get_absolute` 转换，支持 `.` 或指定目录）。
- **`break_locks`**：**TRUE**（强制打破现有锁，解决大多数“working copy locked”问题）。
- **`fix_recorded_timestamps`**：**TRUE**（修复文件时间戳记录）。
- **`clear_dav_cache`**：**TRUE**（清除 DAV 缓存）。
- **`vacuum_pristines`**：**TRUE**（清理未引用 pristine 文件）。
- **`include_externals`**：`opt_state->include_externals`（**默认 FALSE**；只有显式加 `--include-externals` 选项时才为 TRUE）。
- **`ctx`**：客户端上下文（含取消/通知回调）。
- **`scratch_pool`**：临时内存池（`iterpool`）。

### 2. 特殊情况（带以下任意选项时）
- `--remove-unversioned`
- `--remove-ignored`
- `--vacuum-pristines`

代码走 `if` 分支，**不再调用 `svn_client_cleanup2`**，而是改用 `svn_client_vacuum(target_abspath, ...)`（带 `fix_timestamps=TRUE`、`include_externals` 等）。

### 命令行选项映射总结
- `svn cleanup` **只支持** `--include-externals`、`--remove-unversioned`、`--remove-ignored`、`--vacuum-pristines`。
- **没有** `--break-locks`、`--fix-timestamps` 等选项（这些在 API 中暴露，但命令行**硬编码为 TRUE**）。
- `--vacuum-pristines` 选项会切换到 `svn_client_vacuum`，而不是在 `cleanup2` 中生效。

### 源码参考
[cleanup-cmd.c（完整实现）](https://svn.apache.org/viewvc/subversion/trunk/subversion/svn/cleanup-cmd.c?view=co)

这就是 `svn cleanup` 命令**实际**对 `svn_client_cleanup2` 的调用方式（1.9+ 版本）。如果带了移除选项，底层走的是另一个 API。需要更详细的 `svn_client_vacuum` 参数解析或历史版本差异吗？随时说！