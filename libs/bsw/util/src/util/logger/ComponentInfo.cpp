// Copyright 2024 Accenture.

#include "util/logger/ComponentInfo.h"

#ifndef LOGGER_NO_LEGACY_API

#include <estd/assert.h>

#include <type_traits>

namespace util
{
namespace logger
{

// required for rust interop
static_assert(std::is_standard_layout<ComponentInfo>::value, "required for rust interop");
static_assert(
    std::is_standard_layout<ComponentInfo::PlainInfo>::value, "required for rust interop");

ComponentInfo& ComponentInfo::operator=(ComponentInfo const& src)
{
    if (this != &src)
    {
        _componentIndex = src._componentIndex;
        _plainInfo      = src._plainInfo;
    }
    return *this;
}

::util::format::AttributedString ComponentInfo::getName() const
{
    estd_assert(_plainInfo != nullptr);
    return ::util::format::AttributedString(_plainInfo->_nameInfo);
}

} /* namespace logger */
} /* namespace util */

#endif /* LOGGER_NO_LEGACY_API */
