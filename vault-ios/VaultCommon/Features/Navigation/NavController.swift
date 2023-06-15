import Combine
import Foundation

public struct RouteContainer<Route: Equatable & Hashable>: Equatable & Hashable {
    public var id: Int
    public var route: Route

    public init(id: Int, route: Route) {
        self.id = id
        self.route = route
    }
}

public class NavController<Route: Equatable & Hashable>: ObservableObject {
    @Published public var path: [RouteContainer<Route>]
    @Published public var state: State
    public var viewModels: [Int: [String: Any]]

    private var pathChangedCancellable: AnyCancellable?

    public init(rootRoute: Route) {
        path = []
        state = State(rootRoute: rootRoute)
        viewModels = [0: [:]]

        pathChangedCancellable = self.$path.sink { [weak self] path in
            self?.pathChanged(path: path)
        }
    }

    public func push(_ route: Route) {
        print("NavController push route: \(route)")

        var navOpQueue = state.navOpQueue
        navOpQueue.append(.push(route: route))

        state = state.withNavOpQueue(navOpQueue)

        processNavOps()
    }

    public func pop() {
        print("NavController pop")

        var navOpQueue = state.navOpQueue
        navOpQueue.append(.pop)

        state = state.withNavOpQueue(navOpQueue)

        processNavOps()
    }

    public func ensureViewModel<T>(routeContainer: RouteContainer<Route>, create: () -> T) -> T {
        if viewModels[routeContainer.id] == nil {
            // route does not exist anymore, do not cache the view model
            return create()
        }

        let type = String(describing: T.self)

        if let vm = viewModels[routeContainer.id]![type] {
            return vm as! T
        }

        let vm = create()

        viewModels[routeContainer.id]![type] = vm

        return vm
    }

    public func onAppear(_ routeContainer: RouteContainer<Route>) {
        print("NavController onAppear route container: \(routeContainer)")

        var routesState = state.routesState
        routesState[routeContainer.id] = routesState[routeContainer.id]?.withVisible(true)

        state = state.withRoutesState(routesState)

        processNavWait(navWait: .appear(id: routeContainer.id))
    }

    public func onDisappear(_ routeContainer: RouteContainer<Route>) {
        print("NavController onDisappear route container: \(routeContainer)")

        if let routeState = state.routesState[routeContainer.id] {
            let routeState = routeState.withVisible(false)

            var routesState = state.routesState
            routesState[routeContainer.id] = routeState

            checkRouteCleanup(routesState: &routesState, id: routeContainer.id)

            state = state.withRoutesState(routesState)

            processNavWait(navWait: .disappear(id: routeContainer.id))
        }
    }

    public func navigationControllerDidShow() {
        print("NavController did show")

        processNavWait(navWait: .navigationControllerDidShow)
    }

    private func buildRouteContainer(route: Route) -> RouteContainer<Route> {
        let id = state.nextId

        let routeState = RouteState(id: id, route: route, inPath: false, visible: false)

        var routesState = state.routesState

        routesState[id] = routeState

        state = state.withNextId(state.nextId + 1).withRoutesState(routesState)

        viewModels[id] = [:]

        return RouteContainer(id: id, route: route)
    }

    private func processNavOps() {
        while !state.navOpQueue.isEmpty && state.navWait.isEmpty {
            var navOpQueue = state.navOpQueue

            let navOp = navOpQueue.removeFirst()

            state = state.withNavOpQueue(navOpQueue)

            processNavOp(navOp: navOp)
        }
    }

    private func processNavOp(navOp: NavOp) {
        print("NavController process nav op: \(navOp)")

        switch navOp {
        case .push(let route):
            processNavOpPush(route: route)
        case .pop:
            processNavOpPop()
        }
    }

    private func processNavOpPush(route: Route) {
        let routeContainer = buildRouteContainer(route: route)

        print("NavController process nav op push route container: \(routeContainer)")

        var path = state.path
        path.append(routeContainer)

        var navWait = state.navWait
        // new route must appear and old route must disappear before we can continue with navigation
        navWait.append(.appear(id: routeContainer.id))
        navWait.append(.disappear(id: state.activeRouteContainer.id))

        state = state.withPath(path).withNavWait(navWait)

        self.path.append(routeContainer)
    }

    private func processNavOpPop() {
        var path = state.path

        guard let routeContainer = path.popLast() else {
            return
        }

        var navWait = state.navWait
        // the active route must disappear before we can continue with
        // navigation
        navWait.append(.disappear(id: routeContainer.id))

        state = state.withPath(path).withNavWait(navWait)

        self.path.removeLast()
    }

    private func processNavWait(navWait: NavWait) {
        print("NavController process nav wait: \(navWait)")

        if state.navWait.first == navWait {
            var navWait = state.navWait
            navWait.removeFirst()

            state = state.withNavWait(navWait)

            processNavOps()
        }
    }

    private func pathChanged(path: [RouteContainer<Route>]) {
        print("NavController path changed: \(path)")

        var pathRouteIds = Set<Int>()

        pathRouteIds.insert(0)

        for routeContainer in path {
            pathRouteIds.insert(routeContainer.id)
        }

        var routesState = state.routesState

        for id in Array(routesState.keys) {
            // update routeState.inPath
            let routeState = routesState[id]!.withInPath(pathRouteIds.contains(id))

            routesState[id] = routeState

            print("NavController path changed route state: \(routeState)")

            // remove routeState if not inPath and not visible anymore
            checkRouteCleanup(routesState: &routesState, id: id)
        }

        var navWait = state.navWait

        // generate navWait for routeStates that are visible but not in
        // state.path anymore (state.path, not path, from last to root)
        for routeContainer in state.path.reversed() {
            if let routeState = routesState[routeContainer.id] {
                if !routeState.inPath {
                    // the route must disappear before we can continue with
                    // navigation
                    navWait.append(.disappear(id: routeContainer.id))
                }
            }
        }

        state = state.withPath(path).withRoutesState(routesState).withNavWait(navWait)
    }

    private func checkRouteCleanup(routesState: inout [Int: RouteState], id: Int) {
        guard let routeState = routesState[id] else {
            return
        }

        if !routeState.inPath && !routeState.visible {
            print("NavController ckech route cleanup remove route: \(routeState)")

            routesState.removeValue(forKey: id)

            viewModels.removeValue(forKey: id)
        }
    }

    public enum NavOp: Equatable {
        case push(route: Route)
        case pop
    }

    public enum NavWait: Equatable {
        case appear(id: Int)
        case disappear(id: Int)
        case navigationControllerDidShow
    }

    public struct RouteState: Equatable {
        public let id: Int
        public let route: Route
        public let inPath: Bool
        public let visible: Bool

        public init(id: Int, route: Route, inPath: Bool, visible: Bool) {
            self.id = id
            self.route = route
            self.inPath = inPath
            self.visible = visible
        }

        public func withInPath(_ inPath: Bool) -> RouteState {
            return RouteState(id: id, route: route, inPath: inPath, visible: visible)
        }

        public func withVisible(_ visible: Bool) -> RouteState {
            return RouteState(id: id, route: route, inPath: inPath, visible: visible)
        }
    }

    public struct State: Equatable {
        public let path: [RouteContainer<Route>]
        public let rootRouteContainer: RouteContainer<Route>
        public let nextId: Int
        public let routesState: [Int: RouteState]
        public let navOpQueue: [NavOp]
        public let navWait: [NavWait]

        public var activeRouteContainer: RouteContainer<Route> {
            path.last ?? rootRouteContainer
        }

        public var activeRoute: Route {
            activeRouteContainer.route
        }

        public var isNavigating: Bool {
            navOpQueue.count > 0 || navWait.count > 0
                || routesState.values.lazy.filter({ $0.visible }).count > 1
        }

        public init(
            path: [RouteContainer<Route>],
            rootRouteContainer: RouteContainer<Route>,
            nextId: Int,
            routesState: [Int: RouteState],
            navOpQueue: [NavOp],
            navWait: [NavWait]
        ) {
            self.path = path
            self.rootRouteContainer = rootRouteContainer
            self.nextId = nextId
            self.routesState = routesState
            self.navOpQueue = navOpQueue
            self.navWait = navWait
        }

        public init(rootRoute: Route) {
            let path = [RouteContainer<Route>]()

            let rootRouteContainer = RouteContainer(id: 0, route: rootRoute)

            let nextId = 1

            var routesState = [Int: RouteState]()
            routesState[rootRouteContainer.id] = RouteState(
                id: rootRouteContainer.id, route: rootRouteContainer.route, inPath: true,
                visible: false)

            let navOpQueue = [NavOp]()

            var navWait = [NavWait]()
            // wait for the root view to be shown before any navigation is
            // possible
            navWait.append(.navigationControllerDidShow)

            self.init(
                path: path,
                rootRouteContainer: rootRouteContainer,
                nextId: nextId,
                routesState: routesState,
                navOpQueue: navOpQueue,
                navWait: navWait
            )
        }

        public func withPath(_ path: [RouteContainer<Route>]) -> State {
            return State(
                path: path,
                rootRouteContainer: rootRouteContainer,
                nextId: nextId,
                routesState: routesState,
                navOpQueue: navOpQueue,
                navWait: navWait
            )
        }

        public func withNextId(_ nextId: Int) -> State {
            return State(
                path: path,
                rootRouteContainer: rootRouteContainer,
                nextId: nextId,
                routesState: routesState,
                navOpQueue: navOpQueue,
                navWait: navWait
            )
        }

        public func withRoutesState(_ routesState: [Int: RouteState]) -> State {
            return State(
                path: path,
                rootRouteContainer: rootRouteContainer,
                nextId: nextId,
                routesState: routesState,
                navOpQueue: navOpQueue,
                navWait: navWait
            )
        }

        public func withNavOpQueue(_ navOpQueue: [NavOp]) -> State {
            return State(
                path: path,
                rootRouteContainer: rootRouteContainer,
                nextId: nextId,
                routesState: routesState,
                navOpQueue: navOpQueue,
                navWait: navWait
            )
        }

        public func withNavWait(_ navWait: [NavWait]) -> State {
            return State(
                path: path,
                rootRouteContainer: rootRouteContainer,
                nextId: nextId,
                routesState: routesState,
                navOpQueue: navOpQueue,
                navWait: navWait
            )
        }
    }
}
