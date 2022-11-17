use commander::cdr_timer;
use peregrine_toolkit::{plumbing::oneshot::OneShot, log_extra};
use web_sys::HtmlElement;
use crate::{PgCommanderWeb, Message};

/* The shutdown detector listens for an element being disconnected from the DOM. When that
 * happens, the passed OneShot runs which will have shutdown and dreregistering type stuff
 * on it.
 */

async fn check_for_shutdown(oneshot: &OneShot, element: &HtmlElement) -> Result<bool,Message> {
    if !element.is_connected() {
        log_extra!("shutting down");
        oneshot.run();
        Ok(true)
    } else {
        Ok(false)
    }
}

pub(super) fn detect_shutdown(commander: &PgCommanderWeb, oneshot: &OneShot, element: &HtmlElement) ->Result<(),Message> {
    let oneshot = oneshot.clone();
    let element = element.clone();
    commander.add::<Message>("shutdown detector",20,None,None,Box::pin(async move {
        while !check_for_shutdown(&oneshot,&element).await? {
            cdr_timer(5000.).await;
        }
        Ok(())
    }));
    Ok(())
}
