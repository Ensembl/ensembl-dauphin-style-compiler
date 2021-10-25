use crate::{Message, PeregrineInnerAPI, input::translate::{axisphysics::{AxisPhysicsConfig, Scaling}, measure::Measure}, run::{PgConfigKey, PgPeregrineConfig}};
use super::{dragregime::{DragRegime, DragRegimeCreator}, setregime::{SetRegime, SetRegimeCreator}, windowregime::{WRegime, WRegimeCreator}, zoomxregime::{ZoomXRegime, ZoomXRegimeCreator}};

pub(crate) enum TickResult {
    Finished,
    Update(Option<f64>,Option<f64>)
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
    W(WRegime),
    UserPull(DragRegime),
    InstructedPull(DragRegime),
    SelfPull(DragRegime),
    None(RegimeNone),
    ZoomX(ZoomXRegime)
}

impl RegimeObject {
    fn as_trait_mut(&mut self) -> &mut dyn RegimeTrait {
        match self {
            RegimeObject::Set(r) => r,
            RegimeObject::W(r) => r,
            RegimeObject::UserPull(r) => r,
            RegimeObject::InstructedPull(r) => r,
            RegimeObject::SelfPull(r) => r,
            RegimeObject::None(r) => r,
            RegimeObject::ZoomX(r) => r
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
    w_creator: WRegimeCreator,
    user_drag_creator: DragRegimeCreator,
    instructed_drag_creator: DragRegimeCreator,
    self_drag_creator: DragRegimeCreator,
    zoomx_creator: ZoomXRegimeCreator,
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
        let self_drag_config = make_drag_axis_config(config,&PgConfigKey::SelfDragLethargy)?;
        let w_config = make_axis_config(config,&PgConfigKey::WindowLethargy)?;
        let zoomx_config = instructed_drag_config.0.clone();
        instructed_drag_config.0.vel_min *= 100.;
        instructed_drag_config.0.force_min *= 100.;
        Ok(Regime {
            object: RegimeObject::None(RegimeNone()),
            set_creator: SetRegimeCreator(),
            w_creator: WRegimeCreator(w_config),
            user_drag_creator: DragRegimeCreator(user_drag_config.0,user_drag_config.1),
            instructed_drag_creator: DragRegimeCreator(instructed_drag_config.0,instructed_drag_config.1),
            self_drag_creator: DragRegimeCreator(self_drag_config.0,self_drag_config.1),
            zoomx_creator: ZoomXRegimeCreator(zoomx_config),
            size: None
        })
    }

    set_regime!(regime_set,try_regime_set,SetRegime,Set,set_creator);
    set_regime!(regime_w,try_regime_w,WRegime,W,w_creator);
    set_regime!(regime_user_drag,try_regime_user_drag,DragRegime,UserPull,user_drag_creator);
    set_regime!(regime_instructed_drag,try_regime_instructed_drag,DragRegime,InstructedPull,instructed_drag_creator);
    set_regime!(regime_self_drag,try_regime_self_drag,DragRegime,SelfPull,self_drag_creator);
    set_regime!(regime_zoomx,try_regime_zoomx,ZoomXRegime,ZoomX,zoomx_creator);

    pub(crate) fn tick(&mut self, inner: &mut PeregrineInnerAPI, total_dt: f64) -> Result<(),Message> {
        let measure = if let Some(measure) = Measure::new(inner)? { measure } else { return Ok(()); };
        self.update_settings(&measure);
        let (new_x,new_bp) = match self.object.as_trait_mut().tick(&measure,total_dt) {
            TickResult::Update(x,bp) => (x,bp),
            TickResult::Finished => {
                self.object = RegimeObject::None(RegimeNone());
                (None,None)
            }
        };
        if let Some(new_x) = new_x {
            inner.set_x(new_x);
        }
        if let Some(bp_per_screen) = new_bp {
            inner.set_bp_per_screen(bp_per_screen);
        }
        Ok(())
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
