use crate::EcmaAnalyzer;
use rustc_hash::FxHashMap;
use std::mem;

pub struct LoopData<'a, A: EcmaAnalyzer<'a> + ?Sized> {
  call_id: usize,
  tests: Vec<A::Entity>,
}

pub type LoopDataMap<'a, A: EcmaAnalyzer<'a> + ?Sized> = FxHashMap<usize, Vec<LoopData<'a, A>>>;

pub trait LoopScopeAnalyzer<'a> {
  fn post_analyze_handle_loops(&mut self) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    let mut remained = LoopDataMap::default();
    let mut dirty = false;
    for (loop_dep, data) in mem::take(&mut self.loop_data) {
      if self.is_referred(loop_dep) {
        let mut remained_data = vec![];
        for data in data {
          if self.is_referred(data.call_id) {
            self.consume(data.tests);
            dirty = true;
          } else {
            remained_data.push(data);
          }
        }
        if !remained_data.is_empty() {
          remained.insert(loop_dep, remained_data);
        }
      } else {
        remained.insert(loop_dep, data);
      }
    }
    dirty
  }
}
