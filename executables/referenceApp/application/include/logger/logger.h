// Copyright 2024 Accenture.

#pragma once

#include <logger/ComponentMapping.h>

namespace logger
{
::logger::PlainLoggerMappingInfo const* getLoggerComponentInfoTable();
uintptr_t getLoggerComponentInfoTableSize();
void init();
void run();
void flush();

} // namespace logger
