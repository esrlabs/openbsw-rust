// Copyright 2024 Accenture.

#pragma once

#include "async/Async.h"
#include "async/Config.h"
#include "async/Types.h"

#include <cstdint>
#include <cstdio>
extern "C" void runtime_poll(uint8_t context);
extern "C" void check_alarm(uint8_t context);


class ExecutorRunnable final : private ::async::IRunnable
{
public:
    ExecutorRunnable(ExecutorRunnable const&) = delete;
    ExecutorRunnable& operator=(ExecutorRunnable const&) = delete;
    ExecutorRunnable() = delete;
    ExecutorRunnable(::async::ContextType context) : _context(context), _timeout(){}

    void schedulePoll() { ::async::execute(_context, *this); }

    void schedulePollIn(uint32_t delay_us)
    {
        if (getSystemTimeUs32Bit() + delay_us < _timeout._time)
        {
            _timeout.cancel();
        }
        ::async::schedule(_context, *this, _timeout, delay_us, ::async::TimeUnit::MICROSECONDS);
    }

private:
    ::async::ContextType _context;
    ::async::TimeoutType _timeout;
    void execute()
    {
        check_alarm(_context);
        runtime_poll(_context);
    }
};

static ExecutorRunnable DEMO_EXECUTOR_RUNNABLE(TASK_DEMO);
static ExecutorRunnable BACKGROUND_EXECUTOR_RUNNABLE(TASK_BACKGROUND);

extern "C" void schedule_rust_runtime(uint8_t context)
{
    switch (context) {
    case TASK_DEMO:
        DEMO_EXECUTOR_RUNNABLE.schedulePoll();
        break;
    case TASK_BACKGROUND:
        BACKGROUND_EXECUTOR_RUNNABLE.schedulePoll();
        break;
    }
}

extern "C" void schedule_rust_runtime_in(uint8_t context, uint32_t delay_us) {
    switch (context) {
    case TASK_DEMO:
        DEMO_EXECUTOR_RUNNABLE.schedulePollIn(delay_us);
        break;
    case TASK_BACKGROUND:
        BACKGROUND_EXECUTOR_RUNNABLE.schedulePollIn(delay_us);
        break;
    }
 }
