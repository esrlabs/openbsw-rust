// Copyright 2024 Accenture.

#include "app/app.h"

#include "async/Config.h"
#include "async/IRunnable.h"
#include "console/console.h"
#include "estd/typed_mem.h"
#include "logger/logger.h"
#include "reset/softwareSystemReset.h"
#include "systems/DemoSystem.h"
#include "systems/RuntimeSystem.h"
#include "systems/SafetySystem.h"
#include "systems/SysAdminSystem.h"

#include <app/appConfig.h>
#include <async/Async.h>
#include <async/util/Call.h>

#include <cstdint>

#ifdef PLATFORM_SUPPORT_UDS
#include "busid/BusId.h"
#include "systems/TransportSystem.h"
#include "systems/UdsSystem.h"
#endif // PLATFORM_SUPPORT_UDS

#ifdef PLATFORM_SUPPORT_CAN
#include "systems/DoCanSystem.h"
#endif // PLATFORM_SUPPORT_CAN

#include <async/AsyncBinding.h>
#include <lifecycle/LifecycleLogger.h>
#include <lifecycle/LifecycleManager.h>

#ifdef PLATFORM_SUPPORT_CAN
#include <systems/ICanSystem.h>

namespace systems
{
extern ::can::ICanSystem& getCanSystem();
} // namespace systems
#endif

#include "BswLogger.h"
#include "ExecutorRunnable.h"
#include "rustHelloWorld.h"

#include <platform/estdint.h>

namespace platform
{
// TODO: move declaration to header file
extern void platformLifecycleAdd(::lifecycle::LifecycleManager& lifecycleManager, uint8_t level);
} // namespace platform

namespace app
{
using ::util::logger::LIFECYCLE;
using ::util::logger::Logger;

using AsyncAdapter        = ::async::AsyncBinding::AdapterType;
using AsyncRuntimeMonitor = ::async::AsyncBinding::RuntimeMonitorType;
using AsyncContextHook    = ::async::AsyncBinding::ContextHookType;

constexpr size_t MaxNumComponents         = 16;
constexpr size_t MaxNumLevels             = 8;
constexpr size_t MaxNumComponentsPerLevel = MaxNumComponents;

using LifecycleManager = ::lifecycle::declare::
    LifecycleManager<MaxNumComponents, MaxNumLevels, MaxNumComponentsPerLevel>;

char const* const isrGroupNames[ISR_GROUP_COUNT] = {"test"};

AsyncRuntimeMonitor runtimeMonitor{
    AsyncContextHook::InstanceType::GetNameType::create<&AsyncAdapter::getTaskName>(),
    isrGroupNames};

LifecycleManager lifecycleManager{
    TASK_SYSADMIN,
    ::lifecycle::LifecycleManager::GetTimestampType::create<&getSystemTimeUs32Bit>()};

::estd::typed_mem<::systems::RuntimeSystem> runtimeSystem;
::estd::typed_mem<::systems::SysAdminSystem> sysAdminSystem;
::estd::typed_mem<::systems::DemoSystem> demoSystem;
::estd::typed_mem<::systems::SafetySystem> safetySystem;

#ifdef PLATFORM_SUPPORT_UDS
::estd::typed_mem<::transport::TransportSystem> transportSystem;
#endif

#ifdef PLATFORM_SUPPORT_CAN
::estd::typed_mem<::docan::DoCanSystem> doCanSystem;
#endif

#ifdef PLATFORM_SUPPORT_UDS
::estd::typed_mem<::uds::UdsSystem> udsSystem;
#endif

class LifecycleMonitor : private ::lifecycle::ILifecycleListener
{
public:
    explicit LifecycleMonitor(LifecycleManager& manager) { manager.addLifecycleListener(*this); }

    bool isReadyForReset() const { return _isReadyForReset; }

private:
    void lifecycleLevelReached(
        uint8_t const level,
        ::lifecycle::ILifecycleComponent::Transition::Type const transition) override
    {
        if (0 == level)
        {
            _isReadyForReset = true;
        }
    }

private:
    bool _isReadyForReset = false;
};

LifecycleMonitor lifecycleMonitor(lifecycleManager);

void staticInit()
{
    ::logger::init();

    ::console::init();
    ::console::enable();

    ::logger::add_component_info_array(
        ::logger::getLoggerComponentInfoTableSize(), ::logger::getLoggerComponentInfoTable());
    ::logger::map_crate_to_component("rust_hello_world", "CONSOLE");
    ::logger::install_rust_logging();
}

void staticShutdown()
{
    Logger::info(LIFECYCLE, "Lifecycle shutdown complete");
    ::logger::flush();

    softwareSystemReset();
}

struct InitDemoRunntimeRunnable : public async::IRunnable{
    void execute(){
        rustHelloWorld::init_rust_runtime((uint8_t) TASK_DEMO);
    }
};

struct InitBackgroundRuntimeRunnable: public async::IRunnable{
    void execute(){
        rustHelloWorld::init_rust_runtime((uint8_t)TASK_BACKGROUND);
    }
};

void run()
{
    printf("hello\r\n");
    staticInit();
    AsyncAdapter::init();

    /* runlevel 1 */
    ::platform::platformLifecycleAdd(lifecycleManager, 1U);
    lifecycleManager.addComponent(
        "runtime", runtimeSystem.emplace(TASK_BACKGROUND, runtimeMonitor), 1U);
    lifecycleManager.addComponent(
        "safety", safetySystem.emplace(TASK_SAFETY, lifecycleManager), 1U);
    /* runlevel 2 */
    ::platform::platformLifecycleAdd(lifecycleManager, 2U);
    /* runlevel 3 */
    ::platform::platformLifecycleAdd(lifecycleManager, 3U);
    /* runlevel 4 */
#ifdef PLATFORM_SUPPORT_UDS
    lifecycleManager.addComponent("transport", transportSystem.emplace(TASK_UDS), 4U);
#endif

    /* runlevel 5 */
#ifdef PLATFORM_SUPPORT_CAN
    lifecycleManager.addComponent(
        "docan", doCanSystem.emplace(*transportSystem, ::systems::getCanSystem(), TASK_CAN), 5U);
#endif

    /* runlevel 6 */
#ifdef PLATFORM_SUPPORT_UDS
    lifecycleManager.addComponent(
        "uds",
        udsSystem.emplace(lifecycleManager, *transportSystem, TASK_UDS, LOGICAL_ADDRESS),
        6U);
#endif

    /* runlevel 7 */
    lifecycleManager.addComponent(
        "sysadmin", sysAdminSystem.emplace(TASK_SYSADMIN, lifecycleManager), 7U);

    /* runlevel 8 */
    ::platform::platformLifecycleAdd(lifecycleManager, 8U);
    lifecycleManager.addComponent(
        "demo",
        demoSystem.emplace(
            TASK_DEMO,
            lifecycleManager
#ifdef PLATFORM_SUPPORT_CAN
            ,
            ::systems::getCanSystem()
#endif
                ),
        8U);

    lifecycleManager.transitionToLevel(MaxNumLevels);

    // TODO: Hook into lifecycleManager
    static InitBackgroundRuntimeRunnable rtBg;
    static InitDemoRunntimeRunnable rtDemo;
    ::async::execute(TASK_BACKGROUND, rtBg);
    ::async::execute(TASK_DEMO, rtDemo);

    runtimeMonitor.start();
    AsyncAdapter::run();

    while (true)
    {
        ;
    }
}

void idle(AsyncAdapter::TaskContextType& taskContext)
{
    taskContext.dispatchWhileWork();
    ::logger::run();
    ::console::run();
    if (lifecycleMonitor.isReadyForReset())
    {
        staticShutdown();
    }
}

// TODO find a better place for this
extern "C" ::async::ContextType getCurrentTaskContext(){
    auto const context = ::async::AsyncBinding::AdapterType::getCurrentTaskContext();
    return context;
}

using IdleTask = AsyncAdapter::IdleTask<1024 * 2>;
IdleTask idleTask{"idle", AsyncAdapter::TaskFunctionType::create<&idle>()};

using TimerTask = AsyncAdapter::TimerTask<1024 * 1>;
TimerTask timerTask{"timer"};

using UdsTask = AsyncAdapter::Task<TASK_UDS, 1024 * 2>;
UdsTask udsTask{"uds"};

using SysadminTask = AsyncAdapter::Task<TASK_SYSADMIN, 1024 * 2>;
SysadminTask sysadminTask{"sysadmin"};

using CanTask = AsyncAdapter::Task<TASK_CAN, 1024 * 2>;
CanTask canTask{"can"};

using BspTask = AsyncAdapter::Task<TASK_BSP, 1024 * 2>;
BspTask bspTask{"bsp"};

using DemoTask = AsyncAdapter::Task<TASK_DEMO, 1024 * 5>;
DemoTask demoTask{"demo"};

using BackgroundTask = AsyncAdapter::Task<TASK_BACKGROUND, 1024 * 10>;
BackgroundTask backgroundTask{"background"};

using SafetyTask = AsyncAdapter::Task<TASK_SAFETY, 1024 * 2>;
SafetyTask safetyTask{"safety"};

AsyncContextHook contextHook{runtimeMonitor};

} // namespace app
