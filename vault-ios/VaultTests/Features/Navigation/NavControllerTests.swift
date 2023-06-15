import Vault
import XCTest

final class NavControllerTests: XCTestCase {
    func testPushPop() {
        let navController = NavController<TestRoute>(rootRoute: .root)

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootInitial)

        navController.onAppear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootAppear)

        navController.navigationControllerDidShow()

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootDidShow)

        navController.push(.route1)

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Push)

        navController.onAppear(RouteContainer(id: 1, route: .route1))

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Appear)

        navController.onDisappear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1RootDisappear)

        navController.pop()

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1Pop)

        navController.onAppear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1PopRootAppear)

        navController.onDisappear(RouteContainer(id: 1, route: .route1))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1PopDisappear)
    }

    func testPushPopBefireDidShow() {
        let navController = NavController<TestRoute>(rootRoute: .root)

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootInitial)

        navController.onAppear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootAppear)

        navController.navigationControllerDidShow()

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootDidShow)

        navController.push(.route1)

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Push)

        navController.onAppear(RouteContainer(id: 1, route: .route1))

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Appear)

        navController.pop()

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Appear.withNavOpQueue([.pop]))

        navController.onDisappear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1Pop)

        navController.onAppear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1PopRootAppear)

        navController.onDisappear(RouteContainer(id: 1, route: .route1))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1PopDisappear)
    }

    func testPushPopPath() {
        let navController = NavController<TestRoute>(rootRoute: .root)

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootInitial)

        navController.onAppear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootAppear)

        navController.navigationControllerDidShow()

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootDidShow)

        navController.push(.route1)

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Push)

        navController.onAppear(RouteContainer(id: 1, route: .route1))

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Appear)

        navController.onDisappear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1RootDisappear)

        navController.path.removeLast()

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1Pop)

        navController.onAppear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1PopRootAppear)

        navController.onDisappear(RouteContainer(id: 1, route: .route1))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, route1PopDisappear)
    }

    func testPushMultiWaitForDidShow() {
        let navController = NavController<TestRoute>(rootRoute: .root)

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootInitial)

        navController.push(.route1)

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(navController.state, rootInitial.withNavOpQueue([.push(route: .route1)]))

        navController.push(.route2)

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(
            navController.state,
            rootInitial.withNavOpQueue([.push(route: .route1), .push(route: .route2)]))

        navController.onAppear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(navController.path, [])
        XCTAssertEqual(
            navController.state,
            rootAppear.withNavOpQueue([.push(route: .route1), .push(route: .route2)]))

        navController.navigationControllerDidShow()

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Push.withNavOpQueue([.push(route: .route2)]))

        navController.onAppear(RouteContainer(id: 1, route: .route1))

        XCTAssertEqual(navController.path, [RouteContainer(id: 1, route: .route1)])
        XCTAssertEqual(navController.state, route1Appear.withNavOpQueue([.push(route: .route2)]))

        navController.onDisappear(RouteContainer(id: 0, route: .root))

        XCTAssertEqual(
            navController.path,
            [RouteContainer(id: 1, route: .route1), RouteContainer(id: 2, route: .route2)])
        XCTAssertEqual(navController.state, route2Push)

        navController.onAppear(RouteContainer(id: 2, route: .route2))

        XCTAssertEqual(
            navController.path,
            [RouteContainer(id: 1, route: .route1), RouteContainer(id: 2, route: .route2)])
        XCTAssertEqual(navController.state, route2Appear)

        navController.onDisappear(RouteContainer(id: 1, route: .route1))

        XCTAssertEqual(
            navController.path,
            [RouteContainer(id: 1, route: .route1), RouteContainer(id: 2, route: .route2)])
        XCTAssertEqual(navController.state, route2Route1Disappear)
    }

    func testEnsureViewModel() {
        let navController = NavController<TestRoute>(rootRoute: .root)

        navController.onAppear(RouteContainer(id: 0, route: .root))
        navController.navigationControllerDidShow()
        navController.push(.route1)

        let vm1 = navController.ensureViewModel(
            routeContainer: RouteContainer(id: 1, route: .route1),
            create: {
                TestViewModel()
            })
        let vm1Cached =
            navController.viewModels[1]![String(describing: TestViewModel.self)] as! TestViewModel
        XCTAssertEqual(vm1Cached, vm1)

        navController.onAppear(RouteContainer(id: 1, route: .route1))
        navController.onDisappear(RouteContainer(id: 0, route: .root))

        let vm2 = navController.ensureViewModel(
            routeContainer: RouteContainer(id: 1, route: .route1),
            create: {
                TestViewModel()
            })
        XCTAssertEqual(vm2, vm1)

        navController.pop()

        let vm3 = navController.ensureViewModel(
            routeContainer: RouteContainer(id: 1, route: .route1),
            create: {
                TestViewModel()
            })
        XCTAssertEqual(vm3, vm1)

        navController.onAppear(RouteContainer(id: 0, route: .root))

        let vm4 = navController.ensureViewModel(
            routeContainer: RouteContainer(id: 1, route: .route1),
            create: {
                TestViewModel()
            })
        XCTAssertEqual(vm4, vm1)

        navController.onDisappear(RouteContainer(id: 1, route: .route1))

        XCTAssertNil(navController.viewModels[1])

        let vm5 = navController.ensureViewModel(
            routeContainer: RouteContainer(id: 1, route: .route1),
            create: {
                TestViewModel()
            })
        XCTAssertNotEqual(vm5, vm1)

        XCTAssertNil(navController.viewModels[1])

        let vm6 = navController.ensureViewModel(
            routeContainer: RouteContainer(id: 1, route: .route1),
            create: {
                TestViewModel()
            })
        XCTAssertNotEqual(vm6, vm5)
    }

    // MARK: Test data

    enum TestRoute: Hashable {
        case root
        case route1
        case route2
    }

    struct TestViewModel: Equatable {
        var id: String

        init() {
            self.id = UUID().uuidString
        }
    }

    let rootInitial = NavController<TestRoute>.State(
        path: [],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 1,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: false
            )
        ],
        navOpQueue: [],
        navWait: [.navigationControllerDidShow]
    )

    let rootAppear = NavController<TestRoute>.State(
        path: [],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 1,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: true
            )
        ],
        navOpQueue: [],
        navWait: [.navigationControllerDidShow]
    )

    let rootDidShow = NavController<TestRoute>.State(
        path: [],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 1,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: true
            )
        ],
        navOpQueue: [],
        navWait: []
    )

    let route1Push = NavController<TestRoute>.State(
        path: [RouteContainer(id: 1, route: .route1)],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 2,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: true
            ),
            1: NavController.RouteState(
                id: 1,
                route: .route1,
                inPath: true,
                visible: false
            ),
        ],
        navOpQueue: [],
        navWait: [.appear(id: 1), .disappear(id: 0)]
    )

    let route1Appear = NavController<TestRoute>.State(
        path: [RouteContainer(id: 1, route: .route1)],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 2,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: true
            ),
            1: NavController.RouteState(
                id: 1,
                route: .route1,
                inPath: true,
                visible: true
            ),
        ],
        navOpQueue: [],
        navWait: [.disappear(id: 0)]
    )

    let route1RootDisappear = NavController<TestRoute>.State(
        path: [RouteContainer(id: 1, route: .route1)],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 2,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: false
            ),
            1: NavController.RouteState(
                id: 1,
                route: .route1,
                inPath: true,
                visible: true
            ),
        ],
        navOpQueue: [],
        navWait: []
    )

    let route1Pop = NavController<TestRoute>.State(
        path: [],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 2,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: false
            ),
            1: NavController.RouteState(
                id: 1,
                route: .route1,
                inPath: false,
                visible: true
            ),
        ],
        navOpQueue: [],
        navWait: [.disappear(id: 1)]
    )

    let route1PopRootAppear = NavController<TestRoute>.State(
        path: [],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 2,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: true
            ),
            1: NavController.RouteState(
                id: 1,
                route: .route1,
                inPath: false,
                visible: true
            ),
        ],
        navOpQueue: [],
        navWait: [.disappear(id: 1)]
    )

    let route1PopDisappear = NavController<TestRoute>.State(
        path: [],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 2,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: true
            )
        ],
        navOpQueue: [],
        navWait: []
    )

    let route2Push = NavController<TestRoute>.State(
        path: [RouteContainer(id: 1, route: .route1), RouteContainer(id: 2, route: .route2)],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 3,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: false
            ),
            1: NavController.RouteState(
                id: 1,
                route: .route1,
                inPath: true,
                visible: true
            ),
            2: NavController.RouteState(
                id: 2,
                route: .route2,
                inPath: true,
                visible: false
            ),
        ],
        navOpQueue: [],
        navWait: [.appear(id: 2), .disappear(id: 1)]
    )

    let route2Appear = NavController<TestRoute>.State(
        path: [RouteContainer(id: 1, route: .route1), RouteContainer(id: 2, route: .route2)],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 3,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: false
            ),
            1: NavController.RouteState(
                id: 1,
                route: .route1,
                inPath: true,
                visible: true
            ),
            2: NavController.RouteState(
                id: 2,
                route: .route2,
                inPath: true,
                visible: true
            ),
        ],
        navOpQueue: [],
        navWait: [.disappear(id: 1)]
    )

    let route2Route1Disappear = NavController<TestRoute>.State(
        path: [RouteContainer(id: 1, route: .route1), RouteContainer(id: 2, route: .route2)],
        rootRouteContainer: RouteContainer(id: 0, route: .root),
        nextId: 3,
        routesState: [
            0: NavController.RouteState(
                id: 0,
                route: .root,
                inPath: true,
                visible: false
            ),
            1: NavController.RouteState(
                id: 1,
                route: .route1,
                inPath: true,
                visible: false
            ),
            2: NavController.RouteState(
                id: 2,
                route: .route2,
                inPath: true,
                visible: true
            ),
        ],
        navOpQueue: [],
        navWait: []
    )
}
