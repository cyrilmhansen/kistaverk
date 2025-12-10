use crate::state::{AppState, ScheduledTask, SchedulerLog};
use crate::ui::{
    Button as UiButton, Card as UiCard, Column as UiColumn, Section as UiSection, Text as UiText,
    TextInput as UiTextInput, VirtualList as UiVirtualList,
};
use chrono::{DateTime, Local, TimeZone};
use cron::Schedule;
use serde_json::Value;
use std::str::FromStr;
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

fn ts_to_local(ts: i64) -> DateTime<Local> {
    Local
        .timestamp_opt(ts, 0)
        .single()
        .unwrap_or_else(Local::now)
}

pub fn render_scheduler_screen(state: &AppState) -> Value {
    let mut children: Vec<Value> = Vec::new();
    children.push(
        serde_json::to_value(UiText::new("Task Scheduler").size(20.0)).unwrap(),
    );
    children.push(
        serde_json::to_value(UiText::new("Create cron-based tasks that fire actions on a background thread.").size(12.0))
            .unwrap(),
    );

    if let Some(err) = &state.scheduler.last_error {
        children.push(
            serde_json::to_value(UiText::new(&format!("Error: {err}")).size(12.0)).unwrap(),
        );
    }

    let form_children = vec![
        serde_json::to_value(
            UiTextInput::new("scheduler_name")
                .hint("Task name")
                .text(&state.scheduler.form_name)
                .single_line(true)
                .debounce_ms(150),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("scheduler_action")
                .hint("Action ID (e.g., about)")
                .text(&state.scheduler.form_action)
                .single_line(true)
                .debounce_ms(150),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("scheduler_cron")
                .hint("Cron (e.g., */5 * * * *)")
                .text(&state.scheduler.form_cron)
                .single_line(true)
                .debounce_ms(150),
        )
        .unwrap(),
        serde_json::to_value(UiButton::new("Add task", "scheduler_add")).unwrap(),
    ];
    children.push(
        serde_json::to_value(UiCard::new(form_children).title("New task").padding(12)).unwrap(),
    );

    // Active tasks list
    let mut task_items: Vec<Value> = Vec::new();
    for task in &state.scheduler.tasks {
        let status = match task.last_run_epoch {
            Some(ts) => {
                let dt = ts_to_local(ts);
                format!("Last run: {}", dt.format("%Y-%m-%d %H:%M:%S"))
            }
            None => "Never run".to_string(),
        };
        let enabled_label = if task.enabled { "Enabled" } else { "Paused" };
        let item_children: Vec<Value> = vec![
            serde_json::to_value(UiText::new(&format!("{} ({enabled_label})", task.name)).size(14.0)).unwrap(),
            serde_json::to_value(UiText::new(&format!("Action: {}", task.action)).size(12.0)).unwrap(),
            serde_json::to_value(UiText::new(&format!("Cron: {}", task.cron)).size(12.0)).unwrap(),
            serde_json::to_value(UiText::new(&status).size(12.0)).unwrap(),
            serde_json::to_value(
                UiColumn::new(vec![
                    serde_json::to_value(UiButton::new("Run now", &format!("scheduler_run:{}", task.id))).unwrap(),
                    serde_json::to_value(
                        UiButton::new(
                            if task.enabled { "Pause" } else { "Enable" },
                            &format!("scheduler_toggle:{}", task.id),
                        ),
                    )
                    .unwrap(),
                    serde_json::to_value(
                        UiButton::new("Delete", &format!("scheduler_delete:{}", task.id)),
                    )
                    .unwrap(),
                ])
                .padding(4),
            )
            .unwrap(),
        ];
        task_items.push(serde_json::to_value(UiCard::new(item_children).padding(10)).unwrap());
    }
    if task_items.is_empty() {
        task_items.push(
            serde_json::to_value(UiText::new("No scheduled tasks yet.").size(12.0)).unwrap(),
        );
    }
    children.push(
        serde_json::to_value(
            UiSection::new(vec![serde_json::to_value(UiVirtualList::new(task_items).estimated_item_height(110)).unwrap()])
                .title("Tasks"),
        )
        .unwrap(),
    );

    // Logs
    let mut log_items: Vec<Value> = Vec::new();
    for log in state.scheduler.logs.iter().rev().take(50) {
        let dt = ts_to_local(log.timestamp);
        log_items.push(
            serde_json::to_value(
                UiText::new(&format!(
                    "{} – {}",
                    dt.format("%Y-%m-%d %H:%M:%S"),
                    log.message
                ))
                .size(12.0),
            )
            .unwrap(),
        );
    }
    if log_items.is_empty() {
        log_items.push(
            serde_json::to_value(UiText::new("No task executions yet.").size(12.0)).unwrap(),
        );
    }
    children.push(
        serde_json::to_value(
            UiSection::new(vec![serde_json::to_value(UiVirtualList::new(log_items)).unwrap()])
                .title("Recent activity"),
        )
        .unwrap(),
    );

    serde_json::to_value(UiColumn::new(children).padding(16)).unwrap()
}

pub fn apply_scheduler_result(state: &mut AppState, task_id: u32, action: String, fired_at: i64) {
    if let Some(task) = state.scheduler.tasks.iter_mut().find(|t| t.id == task_id) {
        task.last_run_epoch = Some(fired_at);
        task.last_status = Some("fired".into());
    }
    state.scheduler.logs.push(SchedulerLog {
        task_id,
        message: format!("Fired action {}", action),
        timestamp: fired_at,
    });
    if state.scheduler.logs.len() > 200 {
        let excess = state.scheduler.logs.len() - 200;
        state.scheduler.logs.drain(0..excess);
    }
}

pub fn runtime() -> &'static SchedulerRuntime {
    SCHEDULER_RUNTIME.get_or_init(SchedulerRuntime::new)
}

static SCHEDULER_RUNTIME: OnceLock<SchedulerRuntime> = OnceLock::new();
static SCHEDULER_EVENTS: OnceLock<Mutex<Vec<SchedulerEvent>>> = OnceLock::new();

#[derive(Clone)]
struct SchedulerEvent {
    task_id: u32,
    action: String,
    fired_at: i64,
}

enum RuntimeCommand {
    Replace(Vec<ScheduledTask>),
    RunNow(u32),
}

struct RuntimeTask {
    task: ScheduledTask,
    schedule: Schedule,
    next_fire: Option<DateTime<Local>>,
}

pub struct SchedulerRuntime {
    tx: mpsc::Sender<RuntimeCommand>,
}

impl SchedulerRuntime {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<RuntimeCommand>();
        thread::Builder::new()
            .name("kistaverk-scheduler".into())
            .spawn(move || {
                let mut tasks: Vec<RuntimeTask> = Vec::new();
                loop {
                    while let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            RuntimeCommand::Replace(new_tasks) => {
                                tasks = new_tasks
                                    .into_iter()
                                    .filter(|t| t.enabled)
                                    .filter_map(|t| {
                                        Schedule::from_str(&t.cron)
                                            .ok()
                                            .map(|schedule| {
                                                let mut iter = schedule.upcoming(Local);
                                                let next_fire = iter.next();
                                                RuntimeTask {
                                                    task: t,
                                                    schedule,
                                                    next_fire,
                                                }
                                            })
                                    })
                                    .collect();
                            }
                            RuntimeCommand::RunNow(id) => {
                                let now = Local::now();
                                push_event(SchedulerEvent {
                                    task_id: id,
                                    action: "manual_trigger".into(),
                                    fired_at: now.timestamp(),
                                });
                            }
                        }
                    }

                    let now = Local::now();
                    let mut next_sleep_ms: u64 = 1_000;
                    for task in tasks.iter_mut() {
                        if let Some(next_fire) = task.next_fire {
                            if next_fire <= now {
                                push_event(SchedulerEvent {
                                    task_id: task.task.id,
                                    action: task.task.action.clone(),
                                    fired_at: now.timestamp(),
                                });
                                task.next_fire = task.schedule.upcoming(Local).next();
                            } else {
                                let diff = (next_fire - now).num_milliseconds();
                                if diff > 0 {
                                    next_sleep_ms = next_sleep_ms.min(diff as u64);
                                }
                            }
                        }
                    }

                    thread::sleep(Duration::from_millis(next_sleep_ms.clamp(200, 30_000)));
                }
            })
            .expect("failed to spawn scheduler thread");

        Self { tx }
    }

    pub fn sync_tasks(&self, tasks: &[ScheduledTask]) {
        let _ = self.tx.send(RuntimeCommand::Replace(tasks.to_vec()));
    }

    pub fn trigger_now(&self, task_id: u32) {
        let _ = self.tx.send(RuntimeCommand::RunNow(task_id));
    }
}

fn push_event(event: SchedulerEvent) {
    let queue = SCHEDULER_EVENTS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock();
    if let Ok(mut q) = queue {
        q.push(event);
    }
}

pub fn drain_events() -> Vec<(u32, String, i64)> {
    if let Ok(mut q) = SCHEDULER_EVENTS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
    {
        let drained = q.drain(..).collect::<Vec<_>>();
        return drained
            .into_iter()
            .map(|e| (e.task_id, e.action, e.fired_at))
            .collect();
    }
    Vec::new()
}
