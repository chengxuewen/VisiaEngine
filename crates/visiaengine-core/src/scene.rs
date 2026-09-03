//! 场景图 v0：Vec+free-list slab handle + 代际 + 脏标记（架构③"slab 起步不上 ECS"）。

use thiserror::Error;

/// 槽位 + 代际的实体 handle（COPY 语义，失效由代际判定）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityId {
    slot: u32,
    generation: u32,
}

impl EntityId {
    #[must_use]
    pub fn slot(&self) -> u32 {
        self.slot
    }

    #[must_use]
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum CoreError {
    #[error("实体 {entity:?} 不存在或已删除")]
    NotFound { entity: EntityId },
    #[error("实体 {entity:?} 无组件 {component}")]
    MissingComponent {
        entity: EntityId,
        component: &'static str,
    },
}

/// f64 三维坐标（CORE-08：大坐标零损耗；RTC 重基策略留 P1）。
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub scale: f64,
}

impl Transform {
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }
}

/// 组件 v0：单变体 enum（新组件=新变体，穷举匹配下游——比 Any 字典更贴 IR 哲学）。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Component {
    Transform(Transform),
}

impl Component {
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Transform(_) => "Transform",
        }
    }
}

#[derive(Debug)]
struct Slot {
    alive: bool,
    generation: u32,
    component: Option<Component>,
    dirty: bool,
}

/// 场景存储：槽向量 + 空闲栈。O(1) 分配/回收，代际防悬垂。
#[derive(Debug, Default)]
pub struct Scene {
    slots: Vec<Slot>,
    free: Vec<u32>,
}

impl Scene {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 分配实体（优先复用回收槽，代际+1）。
    pub fn spawn(&mut self) -> EntityId {
        if let Some(slot) = self.free.pop() {
            let s = &mut self.slots[slot as usize];
            s.alive = true;
            s.component = None;
            s.dirty = false;
            s.generation += 1;
            EntityId {
                slot,
                generation: s.generation,
            }
        } else {
            let slot = self.slots.len() as u32;
            self.slots.push(Slot {
                alive: true,
                generation: 0,
                component: None,
                dirty: false,
            });
            EntityId {
                slot,
                generation: 0,
            }
        }
    }

    /// 删除并回收槽位；组件与脏标记一并清空。
    pub fn despawn(&mut self, id: EntityId) -> Result<(), CoreError> {
        let s = self.slot_of_mut(id)?;
        s.alive = false;
        s.component = None;
        s.dirty = false;
        self.free.push(id.slot);
        Ok(())
    }

    #[must_use]
    pub fn is_alive(&self, id: EntityId) -> bool {
        self.slot_of(id).is_ok()
    }

    /// 挂载/替换组件（CORE-10 替换语义）。
    pub fn insert(&mut self, id: EntityId, component: Component) -> Result<(), CoreError> {
        self.slot_of_mut(id)?.component = Some(component);
        Ok(())
    }

    /// 读取组件；存活但缺件=MissingComponent（CORE-09）。
    pub fn get(&self, id: EntityId) -> Result<Component, CoreError> {
        let s = self.slot_of(id)?;
        s.component.ok_or(CoreError::MissingComponent {
            entity: id,
            component: "Transform",
        })
    }

    /// 全部存活实体（含无组件者）。
    #[must_use]
    pub fn alive_ids(&self) -> Vec<EntityId> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.alive)
            .map(|(i, s)| EntityId {
                slot: i as u32,
                generation: s.generation,
            })
            .collect()
    }

    /// 标脏：同帧重复标记合并（CORE-06）。
    pub fn mark_dirty(&mut self, id: EntityId) -> Result<(), CoreError> {
        self.slot_of_mut(id)?.dirty = true;
        Ok(())
    }

    /// 取脏清单并清空（CORE-07）。
    pub fn take_dirty(&mut self) -> Vec<EntityId> {
        let mut out = Vec::new();
        for (i, s) in self.slots.iter_mut().enumerate() {
            if s.alive && s.dirty {
                s.dirty = false;
                out.push(EntityId {
                    slot: i as u32,
                    generation: s.generation,
                });
            }
        }
        out
    }

    fn slot_of(&self, id: EntityId) -> Result<&Slot, CoreError> {
        self.slots
            .get(id.slot as usize)
            .filter(|s| s.alive && s.generation == id.generation)
            .ok_or(CoreError::NotFound { entity: id })
    }

    fn slot_of_mut(&mut self, id: EntityId) -> Result<&mut Slot, CoreError> {
        match self.slots.get_mut(id.slot as usize) {
            Some(s) if s.alive && s.generation == id.generation => Ok(s),
            _ => Err(CoreError::NotFound { entity: id }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: CORE-01
    #[test]
    fn create_get_roundtrip() {
        let mut scene = Scene::new();
        let e = scene.spawn();
        scene
            .insert(e, Component::Transform(Transform::identity()))
            .unwrap();
        assert_eq!(
            scene.get(e).unwrap(),
            Component::Transform(Transform::identity())
        );
    }

    // spec: CORE-02
    #[test]
    fn delete_invalidates_handle() {
        let mut scene = Scene::new();
        let e = scene.spawn();
        scene.despawn(e).unwrap();
        assert!(!scene.is_alive(e));
        assert_eq!(
            scene.get(e),
            Err(CoreError::NotFound { entity: e }),
            "get after despawn must be NotFound"
        );
        assert!(
            scene
                .insert(e, Component::Transform(Transform::identity()))
                .is_err()
        );
        assert!(scene.mark_dirty(e).is_err());
    }

    // spec: CORE-03
    #[test]
    fn stale_generation_rejected() {
        let mut scene = Scene::new();
        let old = scene.spawn();
        scene
            .insert(old, Component::Transform(Transform::identity()))
            .unwrap();
        scene.despawn(old).unwrap();
        let fresh = scene.spawn(); // 复用同槽，新代际
        assert_eq!(
            scene.get(old),
            Err(CoreError::NotFound { entity: old }),
            "旧 handle 不得读到新主数据"
        );
        assert!(matches!(
            scene.get(fresh),
            Err(CoreError::MissingComponent { .. })
        ));
    }

    // spec: CORE-04
    #[test]
    fn slot_reuse_after_delete() {
        let mut scene = Scene::new();
        let a = scene.spawn();
        scene.despawn(a).unwrap();
        let b = scene.spawn();
        assert_eq!(a.slot(), b.slot(), "回收槽必须复用");
        assert_eq!(b.generation(), a.generation() + 1, "代际严格+1");
    }

    // spec: CORE-05
    #[test]
    fn iteration_yields_alive_only() {
        let mut scene = Scene::new();
        let a = scene.spawn();
        let b = scene.spawn();
        let c = scene.spawn();
        scene.despawn(b).unwrap();
        let alive = scene.alive_ids();
        assert_eq!(alive.len(), 2);
        assert!(alive.contains(&a) && alive.contains(&c) && !alive.contains(&b));
    }

    // spec: CORE-06
    #[test]
    fn dirty_flag_coalesces() {
        let mut scene = Scene::new();
        let e = scene.spawn();
        scene.mark_dirty(e).unwrap();
        scene.mark_dirty(e).unwrap();
        scene.mark_dirty(e).unwrap();
        assert_eq!(scene.take_dirty(), vec![e], "同帧重复标脏只记一次");
    }

    // spec: CORE-07
    #[test]
    fn take_dirty_clears() {
        let mut scene = Scene::new();
        let e = scene.spawn();
        scene.mark_dirty(e).unwrap();
        assert_eq!(scene.take_dirty(), vec![e]);
        assert!(scene.take_dirty().is_empty(), "取后即清");
    }

    // spec: CORE-08
    #[test]
    fn f64_position_preserved() {
        let mut scene = Scene::new();
        let e = scene.spawn();
        let far = Vec3::new(1e7 + 0.125, -2.5e8, 1.0 / 3.0);
        scene
            .insert(
                e,
                Component::Transform(Transform {
                    position: far,
                    scale: 0.25,
                }),
            )
            .unwrap();
        let got = scene.get(e).unwrap();
        let expected = Component::Transform(Transform {
            position: far,
            scale: 0.25,
        });
        // f64 PartialEq 对非特殊值即位比较；往返损耗会在此暴露
        assert_eq!(got, expected);
    }

    // spec: CORE-09
    #[test]
    fn component_missing_err() {
        let mut scene = Scene::new();
        let e = scene.spawn();
        assert_eq!(
            scene.get(e),
            Err(CoreError::MissingComponent {
                entity: e,
                component: "Transform"
            })
        );
    }

    // spec: CORE-10
    #[test]
    fn insert_replaces_component() {
        let mut scene = Scene::new();
        let e = scene.spawn();
        let first = Transform::identity();
        let second = Transform {
            position: Vec3::new(9.0, 0.0, 0.0),
            scale: 1.0,
        };
        scene.insert(e, Component::Transform(first)).unwrap();
        scene.insert(e, Component::Transform(second)).unwrap();
        assert_eq!(scene.get(e).unwrap(), Component::Transform(second));
    }

    /// spike-2（architecture.md 开工清单）：10 万实体遍历+抽脏预算。
    /// 非门禁——`#[ignore]`，release 人肉跑一次记录 evidence；机器噪声大不设断言。
    #[test]
    #[ignore = "spike：release 手动跑，见 docs/reference/evidence/"]
    fn spike_slab_iter_100k_budget() {
        use std::time::Instant;
        let mut scene = Scene::new();
        let t0 = Instant::now();
        let ids: Vec<EntityId> = (0..100_000).map(|_| scene.spawn()).collect();
        let spawn_t = t0.elapsed();
        let t1 = Instant::now();
        for id in &ids {
            scene.mark_dirty(*id).unwrap();
        }
        let dirty = scene.take_dirty();
        let alive = scene.alive_ids();
        let work_t = t1.elapsed();
        assert_eq!(dirty.len(), 100_000);
        assert_eq!(alive.len(), 100_000);
        println!("spike_slab: spawn100k={spawn_t:?} mark+take+iter={work_t:?}");
    }
}
