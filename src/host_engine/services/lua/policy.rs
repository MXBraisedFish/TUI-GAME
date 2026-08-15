use std::time::Duration;

/// Lua 回调的执行预算类别。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaBudgetKind {
  Load,
  Init,
  HandleEvent,
  Update,
  UpdateFrame,
  Render,
  Save,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LuaExecutionBudget {
  pub max_duration: Duration,
  pub max_instructions: u64,
}

/// 所有 Lua Session 共用的安全策略。策略本身不持有 VM 状态。
#[derive(Clone, Debug)]
pub struct LuaPolicy {
  pub memory_limit_bytes: usize,
  pub source_limit_bytes: usize,
  pub save_limit_bytes: usize,
  pub save_max_depth: usize,
  pub hook_interval: u32,
  load_budget: LuaExecutionBudget,
  callback_budget: LuaExecutionBudget,
}

impl LuaPolicy {
  pub fn balanced() -> Self {
    Self {
      memory_limit_bytes: 32 * 1024 * 1024,
      source_limit_bytes: 1024 * 1024,
      save_limit_bytes: 1024 * 1024,
      save_max_depth: 32,
      hook_interval: 1_000,
      load_budget: LuaExecutionBudget {
        max_duration: Duration::from_millis(100),
        max_instructions: 1_000_000,
      },
      callback_budget: LuaExecutionBudget {
        max_duration: Duration::from_millis(15),
        max_instructions: 200_000,
      },
    }
  }

  pub fn budget(&self, kind: LuaBudgetKind) -> LuaExecutionBudget {
    match kind {
      LuaBudgetKind::Load | LuaBudgetKind::Init | LuaBudgetKind::Save => self.load_budget,
      LuaBudgetKind::HandleEvent
      | LuaBudgetKind::Update
      | LuaBudgetKind::UpdateFrame
      | LuaBudgetKind::Render => self.callback_budget,
    }
  }

  pub(super) fn validate(&self) -> Result<(), String> {
    if self.memory_limit_bytes == 0 {
      return Err("Lua memory limit must be greater than zero".to_string());
    }
    if self.source_limit_bytes == 0 {
      return Err("Lua source limit must be greater than zero".to_string());
    }
    if self.save_limit_bytes == 0 {
      return Err("Lua save limit must be greater than zero".to_string());
    }
    if self.save_max_depth == 0 {
      return Err("Lua save depth must be greater than zero".to_string());
    }
    if self.hook_interval == 0 {
      return Err("Lua hook interval must be greater than zero".to_string());
    }
    for (name, budget) in [
      ("load", self.load_budget),
      ("callback", self.callback_budget),
    ] {
      if budget.max_duration.is_zero() {
        return Err(format!("{name} duration budget must be greater than zero"));
      }
      if budget.max_instructions == 0 {
        return Err(format!(
          "{name} instruction budget must be greater than zero"
        ));
      }
    }
    Ok(())
  }
}

impl Default for LuaPolicy {
  fn default() -> Self {
    Self::balanced()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn balanced_policy_matches_runtime_limits() {
    let policy = LuaPolicy::balanced();
    assert_eq!(policy.memory_limit_bytes, 32 * 1024 * 1024);
    assert_eq!(policy.hook_interval, 1_000);
    assert_eq!(
      policy.budget(LuaBudgetKind::Render),
      LuaExecutionBudget {
        max_duration: Duration::from_millis(15),
        max_instructions: 200_000,
      }
    );
    assert_eq!(
      policy.budget(LuaBudgetKind::Save),
      LuaExecutionBudget {
        max_duration: Duration::from_millis(100),
        max_instructions: 1_000_000,
      }
    );
  }

  #[test]
  fn invalid_policy_cannot_disable_runtime_limits() {
    let mut policy = LuaPolicy::balanced();
    policy.hook_interval = 0;
    assert!(policy.validate().is_err());

    let mut policy = LuaPolicy::balanced();
    policy.memory_limit_bytes = 0;
    assert!(policy.validate().is_err());
  }
}
