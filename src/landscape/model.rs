//! Landscape 的数据模型与登录参数推导。
//!
//! 树上的条目有两种：基础系统（`SAPGUI`）和快捷方式（`Reference`）。
//! 快捷方式通过 `link` 指向基础系统，登录参数要两边合起来才完整。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceKind {
    SapGui,
    Reference,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Service {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub kind: ServiceKind,
    pub systemid: String,
    pub client: String,
    pub user: String,
    pub language: String,
    /// `host:port` 或登录组名
    pub server: String,
    pub msid: String,
    pub routerid: String,
    /// 快捷方式指向的基础系统 uuid
    pub link: String,
    pub reuse: bool,
    /// 跟踪级别，SAP 写成 `-1`（无跟踪）
    pub sncop: String,
    pub winmax: bool,
    pub work_dir: String,
    pub shortcut: bool,
}

impl Default for ServiceKind {
    fn default() -> Self {
        ServiceKind::SapGui
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Router {
    pub uuid: String,
    pub name: String,
    pub router: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageServer {
    pub uuid: String,
    pub name: String,
    pub host: String,
    pub port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreeNode {
    Folder {
        /// 文件夹在 XML 里的 uuid；写回时按它定位，名称不唯一
        uuid: String,
        name: String,
        children: Vec<TreeNode>,
    },
    Entry {
        service_id: String,
        item_id: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Landscape {
    pub services: Vec<Service>,
    pub routers: Vec<Router>,
    pub message_servers: Vec<MessageServer>,
    /// 顶层文件夹（`01` / `02`），`Local` 工作区本身不显示
    pub tree: Vec<TreeNode>,
}

impl Landscape {
    pub fn service(&self, uuid: &str) -> Option<&Service> {
        self.services.iter().find(|service| service.uuid == uuid)
    }

    /// 把快捷方式解析到它指向的基础系统；基础系统直接返回自己。
    pub fn base_of<'a>(&'a self, service: &'a Service) -> Option<&'a Service> {
        if service.kind == ServiceKind::Reference && !service.link.is_empty() {
            return self.service(&service.link).filter(|base| base.kind == ServiceKind::SapGui);
        }
        Some(service)
    }

    pub fn router(&self, router_id: &str) -> Option<&Router> {
        self.routers.iter().find(|router| router.uuid == router_id)
    }

    pub fn message_server(&self, msid: &str) -> Option<&MessageServer> {
        self.message_servers.iter().find(|server| server.uuid == msid)
    }

    /// 由基础系统推导 `-guiparm`。负载均衡、SAProuter、直连三种拼法不同。
    pub fn guiparm(&self, base: &Service) -> String {
        let mut parameter = String::new();

        if let Some(router) = self.router(&base.routerid) {
            parameter.push_str(&router.router);
        }

        match split_host_port(&base.server) {
            Some((host, port)) => {
                parameter.push_str(&format!("/H/{host}/S/{port}"));
            }
            None if !base.msid.is_empty() => {
                if let Some(server) = self.message_server(&base.msid) {
                    let port = if server.port.trim().is_empty() {
                        "3600"
                    } else {
                        server.port.trim()
                    };
                    parameter.push_str(&format!(
                        "/M/{}/S/{port}/G/{}",
                        server.host, base.server
                    ));
                }
            }
            None => {}
        }

        parameter
    }

    /// 服务器展示文本：直连给 `host:port`，负载均衡给登录组名。
    pub fn server_label(&self, base: &Service) -> String {
        if base.server.trim().is_empty() {
            return "—".into();
        }
        base.server.clone()
    }

    /// 由端口推导实例号（`3201` → `01`）；非直连返回空。
    pub fn instance_label(&self, base: &Service) -> String {
        match split_host_port(&base.server) {
            Some((_, port)) => port
                .parse::<u32>()
                .ok()
                .and_then(|port| port.checked_sub(3200))
                .map(|instance| format!("{instance:02}"))
                .unwrap_or_default(),
            None => String::new(),
        }
    }
}

fn split_host_port(value: &str) -> Option<(&str, &str)> {
    let trimmed = value.trim();
    let (host, port) = trimmed.rsplit_once(':')?;
    if host.is_empty() || port.is_empty() || !port.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((host, port))
}
