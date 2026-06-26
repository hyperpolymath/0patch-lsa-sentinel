// SPDX-License-Identifier: MPL-2.0
// Zig 0.15 build script: static C-ABI library + unit tests.
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const mod = b.createModule(.{
        .root_source_file = b.path("src/sentinel_ffi.zig"),
        .target = target,
        .optimize = optimize,
    });

    const lib = b.addLibrary(.{
        .name = "sentinel_ffi",
        .root_module = mod,
        .linkage = .static,
    });
    lib.installHeader(b.path("include/sentinel.h"), "sentinel.h");
    b.installArtifact(lib);

    const tests = b.addTest(.{ .root_module = mod });
    const run_tests = b.addRunArtifact(tests);
    const test_step = b.step("test", "Run Zig FFI unit tests");
    test_step.dependOn(&run_tests.step);
}
