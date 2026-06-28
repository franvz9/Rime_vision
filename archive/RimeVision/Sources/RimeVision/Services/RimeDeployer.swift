import Foundation

final class RimeDeployer {
    static let shared = RimeDeployer()
    private init() {}

    func deploy() {
        DistributedNotificationCenter.default().postNotificationName(
            NSNotification.Name("SquirrelReloadNotification"),
            object: nil
        )
    }

    func sync() {
        DistributedNotificationCenter.default().postNotificationName(
            NSNotification.Name("SquirrelSyncNotification"),
            object: nil
        )
    }
}
