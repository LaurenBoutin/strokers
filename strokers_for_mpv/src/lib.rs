use mpv_client::{mpv_handle, Event, Handle};

#[no_mangle]
extern "C" fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    let client = Handle::from_ptr(handle);

    println!("strokers plugin for MPV ({}) is loaded!", client.name());

    // Properties we care about:
    // - working_directory
    // - path (path to media, could be relative)
    // - time-pos/full (current playback position in milliseconds)
    //   - playback-time/full is similar but clamped to the duration of the file. I don't think we want that
    // - pause

    // client.observe_property(, )

    loop {
        match client.wait_event(-1.) {
            Event::Shutdown => {
                return 0;
            }
            event => {
                println!("Got event: {}", event);
            }
        }
    }
}
