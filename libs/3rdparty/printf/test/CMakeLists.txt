cmake_minimum_required(VERSION 3.9)

enable_language(CXX)

set(test_targets test_suite autotest)

option(TEST_WITH_NON_STANDARD_FORMAT_STRINGS "Include tests using non-standard-compliant format strings?" ON)
# ... don't worry, we'll suppress the compiler warnings for those.

foreach(tgt ${test_targets})
	add_executable(${tgt} "${tgt}.cpp")
	set_target_properties(
		${tgt}
		PROPERTIES
		CXX_STANDARD 11
		CXX_STANDARD_REQUIRED YES
		CXX_EXTENSIONS NO
	)

	if (TEST_WITH_NON_STANDARD_FORMAT_STRINGS)
		target_compile_definitions(${tgt} PRIVATE TEST_WITH_NON_STANDARD_FORMAT_STRINGS)
	endif()
endforeach()

add_executable(aliasing "aliasing.c")
set_target_properties(
	${tgt}
	PROPERTIES
	C_STANDARD 11
	C_STANDARD_REQUIRED YES
	C_EXTENSIONS NO
)
list(APPEND test_targets aliasing)

foreach(tgt ${test_targets})
	if (CMAKE_CXX_COMPILER_ID STREQUAL "MSVC")
		target_compile_options(test_suite PRIVATE /W4)
	elseif (CMAKE_CXX_COMPILER_ID STREQUAL "GNU" OR
			CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
		# lots of warnings and all warnings as errors
		target_compile_options(${tgt} PRIVATE
			-g
			-Wall
			-Wextra
			-pedantic
			-Wundef
			-Wsign-conversion
			-Wuninitialized
			-Wshadow
			-Wunreachable-code
			-Wswitch-default
			-Wswitch
			-Wcast-align
			-Wmissing-include-dirs
			-Winit-self
			-Wdouble-promotion
			-gdwarf-2
			-fno-exceptions
			-ffunction-sections
			-fdata-sections
			-fverbose-asm
			-Wunused-parameter
		)
		if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
			target_compile_options(${tgt} PRIVATE -ffat-lto-objects)
		endif()
	endif()

endforeach()

# These two lines are necessary, since the test suite does not actually use the
# compiled library - it includes the library's source .c file; and that means we
# need to include the generated config.h file.
target_compile_definitions(test_suite PRIVATE PRINTF_INCLUDE_CONFIG_H)
target_include_directories(
	test_suite 
	PRIVATE 
	"${GENERATED_INCLUDE_DIR}"
    "$<BUILD_INTERFACE:${CMAKE_CURRENT_LIST_DIR}/../src/>"
)
add_test(
	NAME "${PROJECT_NAME}.test_suite"
	COMMAND "test_suite" # ${TEST_RUNNER_PARAMS}
)

target_link_libraries(autotest PRIVATE printf)
target_include_directories(autotest PRIVATE "$<BUILD_INTERFACE:${GENERATED_INCLUDE_DIR}>")

add_test(
	NAME "${PROJECT_NAME}.aliasing"
	COMMAND "aliasing"
)
target_link_libraries(aliasing PRIVATE printf)
target_include_directories(aliasing PRIVATE "$<BUILD_INTERFACE:${GENERATED_INCLUDE_DIR}>")




# Not running autotest by default - it's randomized after all.

