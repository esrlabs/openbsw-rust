// Copyright 2024 Accenture.

#pragma once

#include "async/Async.h"
#include "async/Config.h"
#include "async/Types.h"

#include <cstdint>
#include <cstdio>
extern "C" void runtime_poll();
extern "C" void check_alarm();

class ExecutorRunnable final : private ::async::IRunnable
{
    void execute()
    {
        check_alarm();
        runtime_poll();
    }
public:
    ExecutorRunnable(ExecutorRunnable const&) = delete;
    ExecutorRunnable& operator=(ExecutorRunnable const&) = delete;

    ExecutorRunnable() = default;

    void schedulePoll() { ::async::execute(TASK_DEMO, *this); }

    void schedulePollIn(uint32_t delay_us)
    {
        static ::async::TimeoutType timeout;
        if (getSystemTimeUs32Bit() + delay_us < timeout._time)
        {
            timeout.cancel();
        }
        ::async::schedule(TASK_DEMO, *this, timeout, delay_us, ::async::TimeUnit::MICROSECONDS);
    }
};

static ExecutorRunnable EXECUTOR;

extern "C" void schedule_rust_runtime(uint8_t context)
{
    if (context == TASK_DEMO)
    {
        EXECUTOR.schedulePoll();
    }
}

extern "C" void schedule_rust_runtime_in(uint32_t delay_us) { EXECUTOR.schedulePollIn(delay_us); }
