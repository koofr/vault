#import <UIKit/UIKit.h>

@interface UINavigationControllerDelegateProxy : NSProxy

- (instancetype)initWithDelegate:(id<UINavigationControllerDelegate>)delegate forwardTo:(id<UINavigationControllerDelegate>)forward;

@end
