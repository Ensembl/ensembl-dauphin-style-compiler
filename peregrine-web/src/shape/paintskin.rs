use super::consts::{ PR_LOW, PR_DEF };
use crate::webgl::{ Program, Uniform, Attribute, GLArity, Varying, Statement };

pub(crate) enum PaintSkin {
    Colour,
    Spot,
    Texture
}

impl PaintSkin {
    pub fn to_source(&self) -> Program {
        Program::new(
            match self {
                PaintSkin::Colour => vec![
                    Uniform::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity"),
                    Attribute::new(PR_LOW,GLArity::Vec3,"aVertexColour"),
                    Varying::new(PR_LOW,GLArity::Vec3,"vColour"),
                    Statement::new_vertex("vColour = vec3(aVertexColour)"),
                    Statement::new_fragment("gl_FragColor = vec4(vColour,uOpacity)")
                ],
                PaintSkin::Spot => vec![
                    Uniform::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity"),
                    Uniform::new_fragment(PR_LOW,GLArity::Vec3,"uColour"),
                    Statement::new_fragment("gl_FragColor = vec4(uColour,uOpacity)")
                ],
                PaintSkin::Texture => vec![
                    Uniform::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity"),
                    Uniform::new_fragment(PR_DEF,GLArity::Sampler2D,"uSampler"),
                    Attribute::new(PR_LOW,GLArity::Vec2,"aTextureCoord"),
                    Attribute::new(PR_LOW,GLArity::Vec2,"aMaskCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vTextureCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vMaskCoord"),
                    Statement::new_vertex("vTextureCoord = aTextureCoord"),
                    Statement::new_vertex("vMaskCoord = aMaskCoord"),
                    Statement::new_fragment("gl_FragColor = texture2d(uSampler,vTextureCoord)"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragCOlor.a * uOpacity"),
                    Statement::new_fragment("if(texture2D(uSampler,vMaskCoord).r > 0.95) discard")
                ]
            }
        )
    }
}