#![allow(non_local_definitions)]

use anyhow::{Context, Result};
use codec::{ActivatePaneDirection, InputSerial, SendKeyDown};
use mux::tab::PaneEntry;
use regex::Regex;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use termwiz::input::{KeyCode, KeyEvent};
use wezterm_client::client::Client;
use wezterm_gui_subcommands;

use pyo3::{exceptions::PyValueError, prelude::*};

#[pymodule]
fn wezpy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<WeztermClient>()?;
    Ok(())
}

#[pyclass]
#[derive(Clone)]
struct WeztermClient {
    connection: Client,
    compiled_regexs: Arc<async_std::sync::RwLock<BTreeMap<String, Arc<Regex>>>>,
}

impl WeztermClient {
    async fn get_regex(&self, pattern: Option<String>) -> Result<Option<Arc<Regex>>> {
        let Some(pattern) = pattern else {
            return Ok(None);
        };
        {
            let guard = self.compiled_regexs.read().await;
            if let Some(regex_ptr) = guard.get(&pattern) {
                return Ok(Some(regex_ptr.clone()));
            }
        }
        let regex = Regex::new(&pattern).context("Failed to compile regular expression")?;
        let regex_ptr = Arc::new(regex);
        self.compiled_regexs
            .write()
            .await
            .insert(pattern.to_string(), regex_ptr.clone());
        return Ok(Some(regex_ptr));
    }
}

#[pymethods]
impl WeztermClient {
    #[new]
    fn new() -> Self {
        let mut ui = mux::connui::ConnectionUI::new_headless();
        let client = Client::new_default_unix_domain(
            true,
            &mut ui,
            false,
            true,
            wezterm_gui_subcommands::DEFAULT_WINDOW_CLASS,
        )
        .expect("Unable to connect to wezterm, is it installed and running?");

        Self {
            connection: client,
            compiled_regexs: Arc::new(async_std::sync::RwLock::new(BTreeMap::new())),
        }
    }

    fn find_pane<'a>(
        &self,
        py: Python<'a>,
        workspace_pattern: Option<String>,
        tab_pattern: Option<String>,
        title_pattern: Option<String>,
    ) -> Result<&'a PyAny, pyo3::PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let c = client;
            find_pane(&c, workspace_pattern, tab_pattern, title_pattern)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    // 0 Up,
    // 1 Down,
    // 2 Left,
    // 3 Right,
    // 4 Next,
    // 5 Prev,
    fn navigate_up<'a>(&self, py: Python<'a>) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            navigate_dir(&client, 0)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn navigate_down<'a>(&self, py: Python<'a>) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            navigate_dir(&client, 1)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn navigate_left<'a>(&self, py: Python<'a>) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            navigate_dir(&client, 2)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn navigate_right<'a>(&self, py: Python<'a>) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            navigate_dir(&client, 3)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn navigate_next<'a>(&self, py: Python<'a>) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            navigate_dir(&client, 4)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn navigate_previous<'a>(&self, py: Python<'a>) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            navigate_dir(&client, 5)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn current_pane<'a>(&self, py: Python<'a>) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            current_pane(&client)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn focus_pane<'a>(&self, py: Python<'a>, pane_id: usize) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            focus_pane(&client, pane_id)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn write_to_pane<'a>(
        &self,
        py: Python<'a>,
        pane_id: usize,
        data: String,
    ) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            write_to_pane(&client, pane_id, data.bytes().collect())
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn send_enter<'a>(&self, py: Python<'a>, pane_id: usize) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            send_enter(&client, pane_id)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn send_esc<'a>(&self, py: Python<'a>, pane_id: usize) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            send_esc(&client, pane_id)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }

    fn send_paste<'a>(
        &self,
        py: Python<'a>,
        pane_id: usize,
        data: String,
    ) -> Result<&'a PyAny, PyErr> {
        let client = self.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let client = client;
            send_paste(&client, pane_id, data)
                .await
                .map_err(|msg| PyErr::new::<PyValueError, _>(msg.to_string()))
        })
    }
}

async fn current_pane(client: &WeztermClient) -> Result<usize> {
    // Code from wezterm-client::client.rs resolve_pane_id
    let mut clients = client.connection.list_clients().await?.clients;
    clients.retain(|client| client.focused_pane_id.is_some());
    clients.sort_by(|a, b| b.last_input.cmp(&a.last_input));
    if clients.is_empty() {
        anyhow::bail!(
            "--pane-id was not specified and $WEZTERM_PANE
                         is not set in the environment, and I couldn't
                         determine which pane was currently focused"
        );
    }

    Ok(clients[0]
        .focused_pane_id
        .expect("to have filtered out above"))
}

// See PaneDirection for dir code
async fn navigate_dir(client: &WeztermClient, dir: u8) -> Result<bool> {
    // client.send_pdu()
    let pane_id = current_pane(&client).await?;

    client
        .connection
        .activate_pane_direction(ActivatePaneDirection {
            pane_id,
            direction: unsafe { std::mem::transmute(dir) },
        })
        .await
        .context("Faild to send switch focus command")?;

    async_std::task::sleep(Duration::from_millis(10)).await;

    let new_pane = current_pane(&client).await?;
    Ok(new_pane != pane_id)
}

// , paste: &str
async fn focus_pane(client: &WeztermClient, pane_id: usize) -> Result<()> {
    client
        .connection
        .set_focused_pane_id(codec::SetFocusedPane { pane_id })
        .await
        .context("Failed to set pane focus")?;
    Ok(())
}

async fn write_to_pane(client: &WeztermClient, pane_id: usize, data: Vec<u8>) -> Result<()> {
    client
        .connection
        .write_to_pane(codec::WriteToPane { pane_id, data })
        .await
        .context("Unable to write to pane")?;
    Ok(())
}

async fn send_esc(client: &WeztermClient, pane_id: usize) -> Result<()> {
    client
        .connection
        .key_down(SendKeyDown {
            pane_id,
            event: KeyEvent {
                key: KeyCode::Char('\u{1b}'),
                modifiers: termwiz::input::Modifiers::NONE,
            },
            input_serial: InputSerial::now(),
        })
        .await
        .context("Unable to send esc key")?;
    Ok(())
}

async fn send_enter(client: &WeztermClient, pane_id: usize) -> Result<()> {
    client
        .connection
        .key_down(SendKeyDown {
            pane_id,
            event: KeyEvent {
                key: KeyCode::Enter,
                modifiers: termwiz::input::Modifiers::NONE,
            },
            input_serial: InputSerial::now(),
        })
        .await
        .context("Unable to send enter to pane")?;
    Ok(())
}

async fn send_paste(client: &WeztermClient, pane_id: usize, data: String) -> Result<()> {
    client
        .connection
        .send_paste(codec::SendPaste { pane_id, data })
        .await
        .context("Failed to paste")?;
    Ok(())
}

async fn find_pane(
    client: &WeztermClient,
    workspace_pattern: Option<String>,
    tab_pattern: Option<String>,
    title_pattern: Option<String>,
) -> Result<Option<usize>> {
    let [workspace_regex, tab_regex, title_regex] = futures::future::join_all(vec![
        client.get_regex(workspace_pattern),
        client.get_regex(tab_pattern),
        client.get_regex(title_pattern),
    ])
    .await
    .try_into()
    .unwrap();

    let workspace_regex = workspace_regex?;
    let tab_regex = tab_regex?;
    let title_regex = title_regex?;

    let workspace_regex: Option<&Regex> = workspace_regex.as_deref();
    let tab_regex: Option<&Regex> = tab_regex.as_deref();
    let title_regex: Option<&Regex> = title_regex.as_deref();

    let responce = client
        .connection
        .list_panes()
        .await
        .context("Couldn't fetch panes from wezterm")?;

    let mut panes = Vec::with_capacity(10);
    for (root_node, tab_title) in responce.tabs.iter().zip(responce.tab_titles) {
        if tab_regex.map(|rgx| rgx.is_match(&tab_title)) == Some(false) {
            continue;
        }

        panes.clear();
        flatten_panes(root_node, &mut panes);

        let mut panes = panes.iter().peekable();
        if workspace_regex.and_then(|rgx| panes.peek().map(|pane| rgx.is_match(&pane.workspace)))
            == Some(false)
        {
            continue;
        }

        for pane in panes {
            if title_regex.map(|rgx| rgx.is_match(&pane.title)) == Some(false) {
                continue;
            }
            return Ok(Some(pane.pane_id));
        }
    }
    return Ok(None);
}

fn flatten_panes<'a>(node: &'a mux::tab::PaneNode, result: &mut Vec<&'a PaneEntry>) {
    match node {
        mux::tab::PaneNode::Empty => {}
        mux::tab::PaneNode::Split {
            left,
            right,
            node: _,
        } => {
            flatten_panes(left, result);
            flatten_panes(right, result);
        }
        mux::tab::PaneNode::Leaf(pane_entry) => result.push(pane_entry),
    };
}
