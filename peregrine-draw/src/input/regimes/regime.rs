use crate::{Message, PeregrineInnerAPI, input::translate::{axisphysics::{AxisPhysicsConfig, Scaling}, measure::Measure}, run::{PgConfigKey, PgPeregrineConfig}};
use super::{dragregime::{DragRegime, DragRegimeCreator}, setregime::{SetRegime, SetRegimeCreator}, gotoregime::{GotoRegime, GotoRegimeCreator}};

pub(crate) struct TickUpdate {
    pub x: Option<f64>,
    pub bp: Option<f64>,
    pub force_fade: bool
}

pub(crate) enum TickResult {
    Finished,
    Update(TickUpdate)
}

pub(super) trait RegimeCreator {
    type Object;

    fn create(&self) -> Self::Object;
}

pub(crate) trait RegimeTrait {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>);
    fn tick(&mut self, measure: &Measure, total_dt: f64) -> TickResult;
    fn update_settings(&mut self, measure: &Measure);
    fn is_active(&self) -> bool { true }
}

struct RegimeNone();

impl RegimeTrait for RegimeNone {
    fn set_size(&mut self, _measure: &Measure, _size: Option<f64>) {}
    fn tick(&mut self, _measure: &Measure, _total_dt: f64) -> TickResult { TickResult::Finished }
    fn update_settings(&mut self, _measure: &Measure) {}
    fn is_active(&self) -> bool { false }
}

enum RegimeObject {
    Set(SetRegime),
    UserPull(DragRegime),
    None(RegimeNone),
    Goto(GotoRegime)
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for RegimeObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Set(_) => f.debug_tuple("Set").finish(),
            Self::UserPull(_) => f.debug_tuple("UserPull").finish(),
            Self::None(_) => f.debug_tuple("None").finish(),
            Self::Goto(_) => f.debug_tuple("Goto").finish(),
        }
    }
}

impl RegimeObject {
    fn as_trait_mut(&mut self) -> &mut dyn RegimeTrait {
        match self {
            RegimeObject::Set(r) => r,
            RegimeObject::UserPull(r) => r,
            RegimeObject::None(r) => r,
            RegimeObject::Goto(r) => r,
        }
    }
}

fn make_axis_config(config: &PgPeregrineConfig, lethargy_key: &PgConfigKey) -> Result<AxisPhysicsConfig,Message> {
    Ok(AxisPhysicsConfig {
        lethargy: config.get_f64(lethargy_key)?,
        boing: config.get_f64(&PgConfigKey::AnimationBoing)?,
        vel_min: config.get_f64(&PgConfigKey::AnimationVelocityMin)?,
        force_min: config.get_f64(&PgConfigKey::AnimationForceMin)?,
        brake_mul: config.get_f64(&PgConfigKey::AnimationBrakeMul)?,
        min_bp_per_screen: config.get_f64(&PgConfigKey::MinBpPerScreen)?,
        scaling: Scaling::Linear(1.)
    })
}

fn make_drag_axis_config(config: &PgPeregrineConfig, lethargy_key: &PgConfigKey) -> Result<(AxisPhysicsConfig,AxisPhysicsConfig),Message> {
    let x_config = make_axis_config(config,lethargy_key)?;
    let mut z_config = make_axis_config(config,lethargy_key)?;
    z_config.scaling = Scaling::Logarithmic(100.);
    Ok((x_config,z_config))
}

pub(crate) struct Regime {
    object: RegimeObject,
    set_creator: SetRegimeCreator,
    user_drag_creator: DragRegimeCreator,
    goto_creator: GotoRegimeCreator,
    size: Option<f64>
}

macro_rules! set_regime {
    ($call:ident,$try_call:ident,$inner:ty,$branch:tt,$creator:tt) => {
        pub(crate) fn $call(&mut self, measure: &Measure) -> &mut $inner {
            let create = self.$try_call().is_none();
            if create {
                self.object = RegimeObject::$branch(self.$creator.create());
                self.object.as_trait_mut().set_size(measure,self.size);
            }
            self.update_settings(measure);
            self.$try_call().unwrap()
        }

        #[allow(unused)]
        pub(crate) fn $try_call(&mut self) -> Option<&mut $inner> {
            match &mut self.object {
                RegimeObject::$branch(out) => { return Some(out); },
                _ => { return None; }
            }
        }
    };
}

impl Regime {
    pub(crate) fn new(config: &PgPeregrineConfig) -> Result<Regime,Message> {
        let user_drag_config = make_drag_axis_config(config,&PgConfigKey::UserDragLethargy)?;
        let mut instructed_drag_config = make_drag_axis_config(config,&PgConfigKey::InstructedDragLethargy)?;
        let goto_rho_config = config.get_f64(&PgConfigKey::GotoRho)?;
        let goto_v_config = config.get_f64(&PgConfigKey::GotoV)?;
        let goto_max_s_config = config.get_f64(&PgConfigKey::GotoMaxS)?;
        instructed_drag_config.0.vel_min *= 100.;
        instructed_drag_config.0.force_min *= 100.;
        Ok(Regime {
            object: RegimeObject::None(RegimeNone()),
            set_creator: SetRegimeCreator(),
            user_drag_creator: DragRegimeCreator(user_drag_config.0,user_drag_config.1),
            goto_creator: GotoRegimeCreator { rho: goto_rho_config, v: goto_v_config, max_s: goto_max_s_config },
            size: None
        })
    }

    set_regime!(regime_set,try_regime_set,SetRegime,Set,set_creator);
    set_regime!(regime_user_drag,try_regime_user_drag,DragRegime,UserPull,user_drag_creator);
    set_regime!(regime_goto,try_regime_goto,GotoRegime,Goto,goto_creator);

    pub(crate) fn tick(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<bool,Message> {
        let mut finished = false;
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(true); };
        self.update_settings(&measure);
        let update= match self.object.as_trait_mut().tick(&measure,total_dt) {
            TickResult::Update(u) => u,
            TickResult::Finished => {
                self.object = RegimeObject::None(RegimeNone());
                finished = true;
                TickUpdate { x: None, bp: None, force_fade: false }
            }
        };
        if update.force_fade {
            inner.invalidate();
        }
        if update.x.is_some() || update.bp.is_some() {
            inner.set_position(update.x,update.bp,false);
        }
        Ok(finished)
    }

    pub(crate) fn is_active(&mut self) -> bool {
        self.object.as_trait_mut().is_active()
    }

    pub(crate) fn update_settings(&mut self, measure: &Measure) {
        self.object.as_trait_mut().update_settings(measure);
    }

    pub(crate) fn set_size(&mut self, measure: &Measure, size: f64) {
        if let Some(old_size) = self.size {
            if old_size == size { return; }
        }
        self.size = Some(size);
        self.object.as_trait_mut().set_size(measure,self.size);
    }
}
