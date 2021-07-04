use super::super::core::directcolourdraw::{ DirectColourDraw, DirectProgram };
use super::super::core::spotcolourdraw::{ SpotColourDraw, SpotProgram };
use super::super::core::texture::{ TextureDraw, TextureProgram };
use super::geometry::GeometryProgramLink;
use crate::util::enummap::{Enumerable, EnumerableKey};
use crate::webgl::{FlatId, SetFlag};
use crate::webgl::{ ProcessBuilder, SourceInstrs, UniformProto, AttributeProto, GLArity, Varying, Statement, ProgramBuilder, TextureProto };
use peregrine_data::{ DirectColour, Patina, Colour };
use super::consts::{ PR_LOW, PR_DEF };
use crate::util::message::Message;

pub(crate) enum PatinaProgramLink {
    Direct(DirectProgram),
    Spot(SpotProgram),
    Texture(TextureProgram),
    FreeTexture(TextureProgram)
}

impl PatinaProgramLink {
    pub(super) fn make_patina_process(&self, skin: &PatinaProcessName) -> Result<PatinaProcess,Message> {
        Ok(match self {
            PatinaProgramLink::Direct(v) => PatinaProcess::Direct(DirectColourDraw::new(v)?),
            PatinaProgramLink::Texture(v) => PatinaProcess::Texture(TextureDraw::new(v,false)?),
            PatinaProgramLink::FreeTexture(v) => PatinaProcess::FreeTexture(TextureDraw::new(v,true)?),
            PatinaProgramLink::Spot(v) => {
                match skin {
                    PatinaProcessName::Spot(colour) => PatinaProcess::Spot(SpotColourDraw::new(colour,v)?),
                    _ => { return Err(Message::CodeInvariantFailed(format!("unexpected type mismatch, not spot"))); }
                }
            }
        })
    }
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub(crate) enum PatinaProgramName { Direct, Spot, Texture, FreeTexture }

pub(crate) trait PatinaYielder {
    fn name(&self) -> &PatinaProcessName;
    fn make(&mut self, builder: &ProgramBuilder) -> Result<PatinaProgramLink,Message>;
    fn set(&mut self, program: &PatinaProcess) -> Result<(),Message>;
}

impl EnumerableKey for PatinaProgramName {
    fn enumerable(&self) -> Enumerable {
        Enumerable(match self {
            PatinaProgramName::Direct => 0,
            PatinaProgramName::Spot => 1,
            PatinaProgramName::Texture => 2,
            PatinaProgramName::FreeTexture => 3
        },4)
    }
}

impl PatinaProgramName {
    pub(super) fn make_patina_program(&self, builder: &ProgramBuilder) -> Result<PatinaProgramLink,Message> {
        Ok(match self {
            PatinaProgramName::Direct => PatinaProgramLink::Direct(DirectProgram::new(builder)?),
            PatinaProgramName::Spot => PatinaProgramLink::Spot(SpotProgram::new(builder)?),
            PatinaProgramName::Texture => PatinaProgramLink::Texture(TextureProgram::new(builder)?),
            PatinaProgramName::FreeTexture => PatinaProgramLink::FreeTexture(TextureProgram::new(builder)?),
        })
    }

    pub fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(
            match self {
                PatinaProgramName::Direct => vec![
                    AttributeProto::new(PR_LOW,GLArity::Vec4,"aVertexColour"),
                    Varying::new(PR_LOW,GLArity::Vec4,"vColour"),
                    Statement::new_vertex("vColour = aVertexColour"),
                    Statement::new_fragment("gl_FragColor = vColour"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity")
                ],
                PatinaProgramName::Spot => vec![
                    UniformProto::new_fragment(PR_LOW,GLArity::Vec3,"uColour"),
                    Statement::new_fragment("gl_FragColor = vec4(uColour,uOpacity)")
                ],
                PatinaProgramName::Texture => vec![
                    TextureProto::new("uSampler","uSamplerSize"),
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aTextureCoord"),
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aMaskCoord"),
                    UniformProto::new_fragment(PR_DEF,GLArity::Vec2,"uSize"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vTextureCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vMaskCoord"),
                    Statement::new_vertex("vTextureCoord = aTextureCoord"),
                    Statement::new_vertex("vMaskCoord = aMaskCoord"),
                    Statement::new_fragment("gl_FragColor = texture2D(uSampler,vTextureCoord)"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity"),
                    Statement::new_fragment("if(texture2D(uSampler,vMaskCoord).r > 0.995) discard")
                ],
                PatinaProgramName::FreeTexture => vec![
                    TextureProto::new("uSampler","uSamplerSize"),
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aTextureCoord"),
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aMaskCoord"),
                    UniformProto::new_fragment(PR_DEF,GLArity::Vec2,"uSize"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vTextureCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vMaskCoord"),
                    Statement::new_vertex("vTextureCoord = aTextureCoord"),
                    Statement::new_vertex("vMaskCoord = aMaskCoord"),
                    Statement::new_fragment("gl_FragColor = texture2D(uSampler,vec2(
                            (gl_FragCoord.x-vOrigin.x)/uSamplerSize.x+vTextureCoord.x,
                            (vOrigin.y-gl_FragCoord.y)/uSamplerSize.y+vTextureCoord.y))"),
                    SetFlag::new("need-origin"),
                    Statement::new_fragment("lowp vec4 mask = texture2D(uSampler,vec2(
                        (gl_FragCoord.x-vOrigin.x)/uSamplerSize.x+vMaskCoord.x,
                        (vOrigin.y-gl_FragCoord.y)/uSamplerSize.y+vMaskCoord.y))"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity"),
                    Statement::new_fragment("if(mask.r > 0.995) discard")
                ]
            }
        )
    }
}

pub(crate) enum PatinaProcess {
    Direct(DirectColourDraw),
    Spot(SpotColourDraw),
    Texture(TextureDraw),
    FreeTexture(TextureDraw),
    None
}

// TODO texture types

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum PatinaProcessName { Direct, Spot(DirectColour), Texture(FlatId), FreeTexture(FlatId) }

impl PatinaProcessName {
    pub(super) fn get_program_name(&self) -> PatinaProgramName {
        match self {
            PatinaProcessName::Direct => PatinaProgramName::Direct,
            PatinaProcessName::Spot(_) => PatinaProgramName::Spot,
            PatinaProcessName::Texture(_) => PatinaProgramName::Texture,
            PatinaProcessName::FreeTexture(_) => PatinaProgramName::FreeTexture
        }
    }
}

impl PartialOrd for PatinaProcessName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_program_name().enumerable().partial_cmp(&other.get_program_name().enumerable())
    }
}

impl Ord for PatinaProcessName  {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
