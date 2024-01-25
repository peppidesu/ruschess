use std::collections::HashMap;

use notan::prelude::*;

#[derive(AppState)]
pub struct State {    
    pub draw_context: Box<DrawContext>,
}

impl State {
    pub fn new(gfx: &mut Graphics) -> Self {
        let draw_context = DrawContext {
            textures: TextureTable::new(gfx),
        };

        Self {
            draw_context: Box::new(draw_context),
        }
    }
}

pub struct DrawContext {
    pub textures: TextureTable,
}


pub struct TextureTable {
    map: HashMap<TextureId, Texture>,    
}
impl TextureTable {
    pub fn new(gfx: &mut Graphics) -> Self {
        let mut map = HashMap::new();

        macro_rules! load_from_file {
            ($id:expr, $path:expr) => {
                map.insert(
                    $id,
                    gfx.create_texture()
                        .from_image(include_bytes!($path))
                        .build()
                        .unwrap()
                );
            };
            ($( $id:expr, $path:expr );* $(;)?) => {
                $( load_from_file!($id, $path); )*
            };
        }

        load_from_file!(
            TextureId::WhitePawn, "../assets/white-pawn.png";
            TextureId::WhiteRook, "../assets/white-rook.png";
            TextureId::WhiteKnight, "../assets/white-knight.png";
            TextureId::WhiteBishop, "../assets/white-bishop.png";
            TextureId::WhiteQueen, "../assets/white-queen.png";
            TextureId::WhiteKing, "../assets/white-king.png";
            TextureId::BlackPawn, "../assets/black-pawn.png";
            TextureId::BlackRook, "../assets/black-rook.png";
            TextureId::BlackKnight, "../assets/black-knight.png";
            TextureId::BlackBishop, "../assets/black-bishop.png";
            TextureId::BlackQueen, "../assets/black-queen.png";
            TextureId::BlackKing, "../assets/black-king.png";
        );

        Self {
            map,
        }
    }

    pub fn get(&self, id: TextureId) -> &Texture {
        self.map.get(&id).unwrap()
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TextureId {
    WhitePawn,
    WhiteRook,
    WhiteKnight,
    WhiteBishop,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackRook,
    BlackKnight,
    BlackBishop,
    BlackQueen,
    BlackKing,
}