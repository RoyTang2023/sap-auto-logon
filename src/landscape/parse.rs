//! 解析 SAP UI Landscape。
//!
//! 树上只保留快捷方式（`Reference`）：基础系统是快捷方式指向的目标，
//! 只在新建快捷方式时作为选择器出现。文件夹即便没有快捷方式也保留，
//! 因为右键新建的入口在它们身上。

use std::collections::HashMap;

use roxmltree::Node;

use crate::landscape::model::{Landscape, MessageServer, Router, Service, ServiceKind, TreeNode};

pub fn parse(text: &str) -> anyhow::Result<Landscape> {
    let document = roxmltree::Document::parse(text)?;
    let root = document.root_element();
    let mut landscape = Landscape::default();

    // 先收 Services/Routers/Messageservers：`Workspaces` 在 SAP 写出的文件里
    // 排在 `Services` 之前，一遍循环会拿不到 service 类型，树会被清空。
    for child in root.children().filter(|node| node.is_element()) {
        match child.tag_name().name() {
            "Services" => {
                for service in child
                    .children()
                    .filter(|node| node.is_element() && node.tag_name().name() == "Service")
                {
                    landscape.services.push(read_service(service));
                }
            }
            "Routers" => {
                for router in child
                    .children()
                    .filter(|node| node.is_element() && node.tag_name().name() == "Router")
                {
                    landscape.routers.push(Router {
                        uuid: attribute(router, "uuid"),
                        name: attribute(router, "name"),
                        router: attribute(router, "router"),
                    });
                }
            }
            "Messageservers" => {
                for server in child.children().filter(|node| {
                    node.is_element() && node.tag_name().name() == "Messageserver"
                }) {
                    landscape.message_servers.push(MessageServer {
                        uuid: attribute(server, "uuid"),
                        name: attribute(server, "name"),
                        host: attribute(server, "host"),
                        port: attribute(server, "port"),
                    });
                }
            }
            _ => {}
        }
    }

    let kinds = kind_map(&landscape.services);
    for child in root.children().filter(|node| node.is_element()) {
        if child.tag_name().name() != "Workspaces" {
            continue;
        }
        for workspace in child
            .children()
            .filter(|node| node.is_element() && node.tag_name().name() == "Workspace")
        {
            landscape.tree = read_children(workspace, &kinds);
        }
    }

    Ok(landscape)
}

fn kind_map(services: &[Service]) -> HashMap<String, ServiceKind> {
    services
        .iter()
        .map(|service| (service.uuid.clone(), service.kind))
        .collect()
}

fn read_children(node: Node<'_, '_>, kinds: &HashMap<String, ServiceKind>) -> Vec<TreeNode> {
    let mut children = Vec::new();
    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "Node" => {
                let name = attribute(child, "name");
                children.push(TreeNode::Folder {
                    uuid: attribute(child, "uuid"),
                    name,
                    children: read_children(child, kinds),
                });
            }
            "Item" => {
                let service_id = attribute(child, "serviceid");
                if kinds.get(&service_id) == Some(&ServiceKind::Reference) {
                    children.push(TreeNode::Entry {
                        service_id,
                        item_id: attribute(child, "uuid"),
                    });
                }
            }
            _ => {}
        }
    }
    children
}

fn read_service(node: Node<'_, '_>) -> Service {
    let kind = match attribute(node, "type").as_str() {
        "Reference" => ServiceKind::Reference,
        _ => ServiceKind::SapGui,
    };
    Service {
        uuid: attribute(node, "uuid"),
        name: attribute(node, "name"),
        description: attribute(node, "description"),
        kind,
        systemid: attribute(node, "systemid"),
        client: attribute(node, "client"),
        user: attribute(node, "user"),
        language: attribute(node, "language"),
        server: attribute(node, "server"),
        msid: attribute(node, "msid"),
        routerid: attribute(node, "routerid"),
        link: attribute(node, "link"),
        reuse: attribute(node, "reuse") == "1",
        sncop: attribute(node, "sncop"),
        winmax: attribute(node, "winmax") == "1",
        work_dir: attribute(node, "work_dir"),
        shortcut: attribute(node, "shortcut") == "1",
    }
}

fn attribute(node: Node<'_, '_>, name: &str) -> String {
    node.attribute(name).unwrap_or_default().trim().to_string()
}

/// 读文件，并优先使用缓存：源文件 `mtime + size` 没变就不重新解析。
pub fn parse_file(path: &std::path::Path) -> Option<Landscape> {
    if let Some(cached) = crate::core::cache::load_if_fresh(path) {
        return Some(cached);
    }
    let text = std::fs::read_to_string(path).ok()?;
    match parse(&text) {
        Ok(landscape) => Some(landscape),
        Err(error) => {
            log::error!("解析 {} 失败：{error}", path.display());
            None
        }
    }
}
