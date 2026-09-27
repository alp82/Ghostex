//! THROWAWAY: actual sidebar renderer, existing signal fields, no daemon or persistence.
//! CDXC:Sidebar 2026-09-27 DECISION:
//! User wants sessions kept in place, fills only for confirmed human input, and comparisons beside the existing coloured group headers.
use crate::app::native_sidebar::model::{NativeSidebarSession, NativeSidebarSnapshot};
use crate::*;
use serde_json::{Value, json};

pub(crate) fn enabled() -> bool {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .is_some_and(|q| q.contains("prototype=attention"))
}

pub(crate) fn variant() -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|q| {
            q.trim_start_matches('?')
                .split('&')
                .find_map(|p| p.strip_prefix("variant=").map(str::to_owned))
        })
        .filter(|s| matches!(s.as_str(), "A" | "B" | "C" | "D"))
        .unwrap_or_else(|| "B".into())
}

/// The current sidebar can prove an unanswered async question, not the reason for generic attention.
pub(crate) fn signal(session: &NativeSidebarSession) -> Option<(String, u32, bool)> {
    (session
        .details
        .get("attentionPrototype")
        .and_then(Value::as_bool)
        == Some(true)
        && session
            .details
            .get("pendingQuestionCount")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            > 0)
    .then(|| ("Needs you".into(), 0xf472b6, true))
}

fn pending(row: &NativeSidebarSession) -> u64 {
    row.details
        .get("pendingQuestionCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

impl GhostexGpuiApp {
    pub(crate) fn start_attention_prototype(&mut self, cx: &mut Context<Self>) {
        // Positive-only fixtures: async questions are currently Codex; shell detection is Claude.
        let rows = [
            ("Sidebar approach", "codex", "attention", 1, false),
            ("Keyboard navigation", "codex", "working", 1, false),
            ("Restore sessions", "codex", "attention", 0, false),
            ("Preview server", "claude", "idle", 0, true),
            ("API design question", "codex", "attention", 1, false),
            ("Usage dashboard", "claude", "working", 0, false),
            ("Documentation", "claude", "idle", 0, false),
            ("Refactor commands", "claude", "attention", 0, false),
        ];
        let groups: Vec<Value> = ["Ghostex", "Analytics"].iter().enumerate().map(|(g,title)| {
            let sessions: Vec<Value> = rows.iter().enumerate().skip(g*4).take(4).map(|(i,(title,agent,activity,questions,bg))| json!({
                "sessionId":format!("prototype-{i}"),"alias":title,"activity":activity,"hasBackgroundWork":bg,
                "isFocused":i==2,"isVisible":i==2,"lifecycleState":"running","pendingQuestionCount":questions,
                "attentionPrototype":true,"lastInteractionLabel":"2m","titleTooltip":title,
                "agentIcon":agent,"agentLogoDataUrl":ghostex_gx_core::colored_agent_logo(agent),"menu":[],"agentName":agent
            })).collect();
            json!({"groupId":format!("prototype-group-{g}"),"storageId":format!("prototype-group-{g}"),"summary":{},"collapsed":false,"expanded":true,"hiddenSessionCount":0,"showListToggle":false,"hoverActionsExpanded":false,"menu":[],"headerActions":[],"title":title,"isActive":g==0,"projectContext":{"projectId":format!("prototype-group-{g}")},"sections":[{"id":"sessions","collapsed":false,"count":4,"containsActiveSession":g==0,"workingCount":0,"attentionCount":0,"backgroundWorkCount":0,"questionCount":0,"sessionIds":sessions.iter().map(|s|s["sessionId"].clone()).collect::<Vec<_>>()}],"sessions":sessions})
        }).collect();
        let collections: Vec<Value> = [("Product", "#a78bfa"),("Experiments", "#f472b6")].iter().enumerate().map(|(i,(title,color))|json!({
            "collectionId":format!("collection-{i}"),"title":title,"color":color,"groupIds":[format!("prototype-group-{i}")],"collapsed":false,"containsActiveSession":i==0,"workingCount":0,"attentionCount":0,"backgroundWorkCount":0,"awakeCount":4,"menu":[]
        })).collect();
        let mut snapshot: NativeSidebarSnapshot = serde_json::from_value(json!({
            "scrollScope":"attention-prototype","emptyState":{},"hud":{"settings":{"sidebarTheme":"plain-dark"},"agentManagerZoomPercent":100},
            "groups":groups,"selectedMachineId":"local","machines":[{"id":"local","label":"This computer","state":"connected","workingCount":0,"attentionCount":0,"backgroundWorkCount":0}],
            "spacesEnabled":true,"spaces":[{"id":"personal","name":"Personal","icon":"home","color":"#95d7f6","selected":true,"containsActiveSession":true,"workingCount":0,"attentionCount":0,"backgroundWorkCount":0}],"collections":collections,"order":[{"kind":"collection","id":"collection-0"},{"kind":"collection","id":"collection-1"}],"moreMenu":[]
        })).unwrap();
        refresh_counts(&mut snapshot);
        self.install_native_sidebar_snapshot(Arc::new(snapshot), cx);
    }

    pub(crate) fn attention_prototype_command(&mut self, command: &Value, cx: &mut Context<Self>) {
        let mut snapshot = (**self.native_sidebar.snapshot.as_ref().unwrap()).clone();
        if command["type"] == "selectSession" {
            for group in &mut snapshot.groups {
                for row in &mut group.sessions {
                    let row = Arc::make_mut(row);
                    row.is_focused = command["sessionId"].as_str() == Some(row.session_id.as_str());
                    row.is_visible = row.is_focused;
                }
            }
        } else if command["type"] == "collectionAction" && command["action"] == "toggle" {
            for group in &mut snapshot.collections {
                if command["collectionId"].as_str() == Some(group.collection_id.as_str()) {
                    group.collapsed = !group.collapsed;
                }
            }
        }
        refresh_counts(&mut snapshot);
        self.install_native_sidebar_snapshot(Arc::new(snapshot), cx);
    }

    pub(crate) fn render_attention_prototype(&self, cx: &mut Context<Self>) -> AnyElement {
        let snapshot = self.native_sidebar.snapshot.as_ref().unwrap();
        let row = snapshot
            .groups
            .iter()
            .flat_map(|g| &g.sessions)
            .find(|s| s.is_focused)
            .unwrap();
        let codex = row.agent_icon.as_deref() == Some("codex");
        let controls: &[&str] = if codex {
            &[
                "Question",
                "Question + working",
                "Working",
                "Attention",
                "Idle",
            ]
        } else {
            &["Working", "Attention", "Background detected", "Idle"]
        };
        div().p(px(36.)).flex().flex_col().gap(px(18.)).text_color(rgb(0xb4b8c0))
            .child(div().text_size(px(11.)).child("THROWAWAY PROTOTYPE / ACTUAL GPUI SIDEBAR"))
            .child(div().text_size(px(24.)).child("Fill only when we know you’re needed"))
            .child("Coloured groups use the real group renderer. Sessions stay in place at their normal size.")
            .child("A · Soft fill       B · Strong fill       C · Solid plum + label       D · Neutral fill + label")
            .child(gpui_component::h_flex().gap(px(8.)).children(["A", "B", "C", "D"].map(|key|div().id(key).px(px(16.)).py(px(8.)).rounded(px(5.)).bg(rgb(if variant()==key {0x454b56} else {0x292d35})).cursor_pointer().child(key).on_click(cx.listener(move |_,_,_,_| {let _=web_sys::window().unwrap().location().set_search(&format!("?prototype=attention&variant={key}"));})))))
            .child("Only a positive pending-question count gets a fill. Generic attention keeps its blue dot.")
            .child("No inferred approval / finished / stopped / failed labels. Idle does not prove successful completion.")
            .child("Question counts currently come from Codex async questions; background detection is Claude-specific.")
            .child(div().text_size(px(16.)).child(format!("Selected: {} ({})", row.title(), row.agent_icon.as_deref().unwrap_or(""))))
            .child(div().flex().flex_wrap().gap(px(8.)).children(controls.iter().map(|&label|div().id(label).px(px(12.)).py(px(8.)).rounded(px(5.)).bg(rgb(0x292d35)).cursor_pointer().child(label).on_click(cx.listener(move |app,_,_,cx|{
                let mut snapshot=(**app.native_sidebar.snapshot.as_ref().unwrap()).clone();
                for group in &mut snapshot.groups {for row in &mut group.sessions {if row.is_focused {
                    let row=Arc::make_mut(row);
                    row.activity=match label {"Working"|"Question + working"=>"working","Idle"|"Background detected"=>"idle",_=>"attention"}.into();
                    row.has_background_work=label=="Background detected";
                    row.details.insert("pendingQuestionCount".into(),json!(if label.starts_with("Question"){1}else{0}));
                }}}
                refresh_counts(&mut snapshot);
                app.install_native_sidebar_snapshot(Arc::new(snapshot),cx);
            })))))
            .child("Compare: a waiting question, a question while working, generic attention, background activity and idle.")
            .child(div().flex().flex_col().gap(px(5.)).text_size(px(11.)).children(snapshot.groups.iter().flat_map(|g| &g.sessions).map(|s| div().child(format!("{} | {} | questions: {} | background: {} | selected: {}", s.title(), s.activity, pending(s), s.has_background_work, s.is_focused)))))
            .into_any_element()
    }
}

fn refresh_counts(snapshot: &mut NativeSidebarSnapshot) {
    let mut totals = (0, 0, 0);
    for group in &mut snapshot.groups {
        let working = group
            .sessions
            .iter()
            .filter(|s| s.activity == "working")
            .count();
        let attention = group
            .sessions
            .iter()
            .filter(|s| s.activity == "attention" || pending(s) > 0)
            .count();
        let background = group
            .sessions
            .iter()
            .filter(|s| s.has_background_work)
            .count();
        group.summary = json!({"workingCount":working,"attentionCount":attention,"backgroundWorkCount":background,"awakeCount":group.sessions.len()});
        for section in &mut group.sections {
            section.working_count = working;
            section.attention_count = group
                .sessions
                .iter()
                .filter(|s| s.activity == "attention" && pending(s) == 0)
                .count();
            section.question_count = group.sessions.iter().filter(|s| pending(s) > 0).count();
            section.background_work_count = background;
            section.contains_active_session = group.sessions.iter().any(|s| s.is_focused);
        }
        totals.0 += working;
        totals.1 += attention;
        totals.2 += background;
    }
    for collection in &mut snapshot.collections {
        let groups: Vec<_> = snapshot
            .groups
            .iter()
            .filter(|g| collection.group_ids.contains(&g.group_id))
            .collect();
        collection.working_count = groups
            .iter()
            .map(|g| g.summary["workingCount"].as_u64().unwrap() as usize)
            .sum();
        collection.attention_count = groups
            .iter()
            .map(|g| g.summary["attentionCount"].as_u64().unwrap() as usize)
            .sum();
        collection.background_work_count = groups
            .iter()
            .map(|g| g.summary["backgroundWorkCount"].as_u64().unwrap() as usize)
            .sum();
        collection.contains_active_session = groups
            .iter()
            .any(|g| g.sessions.iter().any(|s| s.is_focused));
    }
    for space in &mut snapshot.spaces {
        space.working_count = totals.0;
        space.attention_count = totals.1;
        space.background_work_count = totals.2;
    }
    for machine in &mut snapshot.machines {
        machine.working_count = totals.0;
        machine.attention_count = totals.1;
        machine.background_work_count = totals.2;
    }
}
