use serde::Serialize;
use tauri::{
    ipc::{Channel, CommandItem, InvokeError, JavaScriptChannelId, Response},
    Runtime,
};

use crate::db::messagepack_serialize;
use crate::error::{builder::General as GeneralSnafu, Error, Result};

/// 将 Result<T, E> 转换为 MessagePack Response
///
/// - Ok(v): 序列化 v 为 MessagePack 并返回
/// - Err(e): 转换为 Error（要求 E 实现 Into<Error>）
pub fn result_to_messagepack_response<T: serde::Serialize, E: Into<Error>>(
    result: Result<T, E>,
) -> Result<Response> {
    match result {
        Ok(v) => to_messagepack_response(v),
        Err(e) => Err(e.into()),
    }
}

/// 将任意 Serialize 值转换为 MessagePack Response
///
pub fn to_messagepack_response<T: serde::Serialize>(value: T) -> Result<Response> {
    let bytes = messagepack_serialize(&value)?;
    Ok(Response::new(bytes))
}

/// 通过 Tauri Channel 发送 MessagePack 编码的消息
///
/// 用法：后端在 `#[messagepack_command]` 函数中拿到 `Channel<Vec<u8>>`，
/// 用此函数把数据序列化为 MessagePack 后通过通道发送给前端 `MessagePackChannel`。
///
/// ## 示例
///
/// ```rust ignore
/// use tauri::ipc::Channel;
///
/// #[messagepack_command]
/// fn subscribe(channel: Channel<Vec<u8>>) {
///     let event = MyEvent { msg: "hello".to_string() };
///     messagepack_channel_send(&channel, event)?;
///     Ok(())
/// }
/// ```
pub fn messagepack_channel_send<T: Serialize>(channel: &Channel<Vec<u8>>, value: &T) -> Result<()> {
    let bytes = messagepack_serialize(value)?;
    channel.send(bytes).map_err(|e| {
        GeneralSnafu {
            detail: format!("MessagePack channel send error: {}", e),
        }
        .build()
    })?;
    Ok(())
}

/// MessagePack 通道 —— 对 `Channel<Vec<u8>>` 的封装。
///
/// 直接用作 `#[messagepack_command]` 的参数类型，内部自动完成：
/// 1. 从 `__CHANNEL__:id` 解析出 Tauri Channel
/// 2. `.send(value)` 时自动用 MessagePack 序列化
///
/// ## 对比
///
/// | 原生用法 | MessagePackChannel 用法 |
/// |---------|-----------------|
/// | `channel: Channel<Vec<u8>>` | `channel: MessagePackChannel` |
/// | `messagepack_channel_send(&channel, event)?` | `channel.send(event)?` |
///
/// ## 示例
///
/// ```rust ignore
/// use svnexus_lib::messagepack_command::MessagePackChannel;
///
/// #[messagepack_command]
/// fn subscribe(#[channel] channel: MessagePackChannel) -> Result<()> {
///     let event = MyEvent { msg: "hello".to_string() };
///     channel.send(event)?;
///     Ok(())
/// }
/// ```
#[derive(derive_more::Debug)]
pub struct MessagePackChannel(#[debug(skip)] pub Channel<Vec<u8>>);

impl MessagePackChannel {
    /// 根据前端序列化的 `__CHANNEL__:id` 创建后端 Channel。
    pub fn from_id<R: Runtime>(webview: &tauri::Webview<R>, value: String) -> Result<Self> {
        let id = value.parse::<JavaScriptChannelId>().map_err(|error| {
            GeneralSnafu {
                detail: format!("Invalid channel value: {error}"),
            }
            .build()
        })?;

        Ok(Self(id.channel_on::<R, Vec<u8>>(webview.clone())))
    }

    /// 发送 MessagePack 编码的消息。
    pub fn send<T: Serialize>(&self, value: &T) -> Result<()> {
        messagepack_channel_send(&self.0, value)
    }

    /// 获取内部 Channel 的 id（与前端 `MessagePackChannel.id` 一致）
    pub fn id(&self) -> u32 {
        self.0.id()
    }
}

// 全部委托给内部 Channel 已有实现

impl Clone for MessagePackChannel {
    fn clone(&self) -> Self {
        MessagePackChannel(self.0.clone())
    }
}

impl Serialize for MessagePackChannel {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// 实现 `CommandArg`，使 `MessagePackChannel` 可以直接作为 `#[messagepack_command]` 的参数。
/// 直接委托给 `Channel<Vec<u8>>` 已有的 `CommandArg` 实现，自己只做一层包装。
impl<'de, R: Runtime> tauri::ipc::CommandArg<'de, R> for MessagePackChannel {
    fn from_command(command: CommandItem<'de, R>) -> std::result::Result<Self, InvokeError> {
        Channel::<Vec<u8>>::from_command(command).map(MessagePackChannel)
    }
}

#[cfg(test)]
mod tests {
    use super::MessagePackChannel;
    use crate::error::Result;

    // Compile-time coverage for the macro's channel parameter expansion.
    #[allow(dead_code)]
    #[svnexus_macro::messagepack_command]
    fn channel_parameter(#[channel] channel: MessagePackChannel) -> Result<()> {
        let _ = channel;
        Ok(())
    }

    #[allow(dead_code)]
    #[svnexus_macro::messagepack_command]
    async fn async_channel_parameter(#[channel] channel: MessagePackChannel) -> Result<()> {
        let _ = channel;
        Ok(())
    }
}
