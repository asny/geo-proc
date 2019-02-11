
use wasm_bindgen::prelude::*;

include!("../../stitching.rs");

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue>
{
    main();
    Ok(())
}
