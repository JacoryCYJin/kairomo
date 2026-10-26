#import <AppKit/AppKit.h>

// NSWorkspace retains the block observer for the process lifetime. Registration
// occurs once on Tauri's main thread; the callback only pauses and persists time.
void kairomo_observe_sleep(void (*callback)(void)) {
    NSNotificationCenter *center = [[NSWorkspace sharedWorkspace] notificationCenter];
    [center addObserverForName:NSWorkspaceWillSleepNotification
                      object:nil
                       queue:nil
                  usingBlock:^(NSNotification *notification) {
        (void)notification;
        callback();
    }];
}
