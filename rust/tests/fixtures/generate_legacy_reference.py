#!/usr/bin/env python3
"""Freeze source-derived master behavior without modifying or executing C#.

Default mode checks the committed fixture. Pass --write deliberately to regenerate.
Numeric values are independently calculated binary32 *approximations*, not C# runtime
goldens. T031 must compare against actual MathF execution and adopt a reviewed tolerance.
"""
import argparse
import hashlib
import json
import math
import struct
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
REV = "ec1af8f8d26061958a19e19e1f66c8c3a41c9f0d"
FORM = "YUAUY_Wallpaper/Form1.cs"
PICKER = "YUAUY_Wallpaper/Image/ImagePicker.cs"
TARGET = Path(__file__).with_name("legacy_reference.json")


def f32(number):
    return struct.unpack("<f", struct.pack("<f", float(number)))[0]


def source(path):
    data = subprocess.check_output(["git", "show", f"{REV}:{path}"], cwd=ROOT)
    return data.decode("utf-8-sig"), hashlib.sha256(data).hexdigest()


def require(text, needle):
    if needle not in text:
        raise ValueError(f"reference expression absent from master: {needle}")
    return needle


def choose_phase(t):
    if t >= 0.75:
        return "new_full"
    if t >= 0.5:
        return "new_visible_white_cover_slides"
    if t >= 0.25:
        return "white_reveal_only"
    return "old_only"


PI32 = f32(math.pi)


def sample_curve(t):
    # Match source operation groupings with per-operation float32 rounding as
    # closely as possible without claiming bitwise parity with .NET MathF.Cosh.
    t = f32(t)
    arg_a = f32(f32(f32(t * f32(3.0)) * PI32) + f32(0.65))
    time_a = f32(f32(1) - f32(f32(1.2) / f32(math.cosh(arg_a))))
    scale = f32(f32(1) - f32(f32(0.2) * time_a))
    phase = choose_phase(t)
    white_curve = None
    if phase == "white_reveal_only":
        b = f32(f32(t * f32(4)) - f32(1))
        arg = f32(f32(f32(b - f32(1)) * f32(2)) * PI32)
        white_curve = f32(f32(1) / f32(math.cosh(arg)))
    if phase == "new_visible_white_cover_slides":
        b = f32(f32(t * f32(4)) - f32(2))
        arg = f32(f32(b * f32(2)) * PI32)
        white_curve = f32(f32(1) - f32(f32(1) / f32(math.cosh(arg))))
    return {
        "time_input": t,
        "phase": phase,
        "time_a_approx": time_a,
        "old_scale_approx": scale,
        "old_cover_alpha_trunc": int(f32(time_a * f32(175))),
        "old_dx_for_x_100_approx": f32(f32(100) * time_a),
        "old_dy_for_y_minus_50_approx": f32(f32(-50) * time_a),
        "old_rotation_for_degrees_19_approx": f32(f32(19) * time_a),
        "white_curve_approx": white_curve,
    }


def image_geom(image_w, image_h, screen_w, screen_h, left, top):
    irs = f32(f32(image_w) / f32(image_h))
    srs = f32(f32(screen_w) / f32(screen_h))
    # C#: Math.Abs(float) returns float; 0.1 and 0.8 are double literals.
    rsx = abs(f32(irs - srs)) * 0.1 + 0.8
    if irs > srs:
        width = int(screen_w * rsx) + 20
        height = int(screen_w * rsx / image_w * image_h) + 20
        side = "wider"
    else:
        width = int(screen_h * rsx / image_h * image_w) + 20
        height = int(screen_h * rsx) + 20
        side = "taller_or_equal"
    return dict(image=[image_w, image_h], monitor={"left": left, "top": top, "width": screen_w, "height": screen_h},
                image_aspect_f32=irs, monitor_aspect_f32=srs,
                scale_rsx_approx=rsx, branch=side, framed_bitmap=[width, height],
                center=[left + screen_w // 2, top + screen_h // 2],
                white_border_pixels=10, draw_bounds=[10,10,width-20,height-20])


def weighted_case(monitors, draws):
    areas = [float(w*h) for w,h in monitors]
    s = sum(areas)
    normalized = [a/s for a in areas]
    selections = []
    for d in draws:
        st = d
        selected = 0
        for prob in normalized:
            st -= prob
            if st <= 0:
                break
            selected += 1
        selections.append({"draw": d, "index": selected})
    return dict(monitors=[list(x) for x in monitors], pixel_areas=areas, weights=normalized, selections=selections)


def make():
    form, form_sha = source(FORM)
    picker, picker_sha = source(PICKER)
    expressions = {
        "area_selection": require(form, "s = s.Select(size => size / s.Sum()).ToArray();"),
        "screen_choice": require(form, "st -= t;"),
        "image_aspect": require(form, "var irs = (float)img.Width / img.Height;"),
        "screen_aspect": require(form, "var srs = (float)Screen.AllScreens[nextScreen].Bounds.Width / Screen.AllScreens[nextScreen].Bounds.Height;"),
        "image_scale": require(form, "var rsx = Math.Abs(irs - srs) * 0.1 + 0.8;"),
        "time_a": require(form, "float timeA = 1 - 1.2f / MathF.Cosh(time * 3f * MathF.PI + 0.65f);"),
        "old_scale": require(form, "var rrsst = 1 - rrss * timeA;"),
        "old_alpha": require(form, "brushA.Color = Color.FromArgb((int)(timeA * 175), 0, 0, 0);"),
        "white_reveal_a": require(form, "timeB = 1 / MathF.Cosh((timeB - 1) * 2f * MathF.PI);"),
        "white_reveal_b": require(form, "timeB = 1 - 1 / MathF.Cosh(timeB * 2f * MathF.PI);"),
        "random_rotation": require(form, "var r = random.Next(-20, 20);"),
        "random_x": require(form, "float x = random.Next((int)(Screen.AllScreens[nextScreen].Bounds.Width * 0.4));"),
        "time_step": require(form, "time += first ? 1 : 0.03f"),
        "frame_delay": require(form, "int delay = 40;"),
        "picker_sequence": require(picker, "if (_index >= _imagePaths.Length) _index = 0;"),
        "picker_rescan": require(picker, "_lastFileCount != ImageFolderList.Sum(path => Directory.GetFiles(path).Length)"),
        "picker_shuffle": require(picker, "imagePathsTmp = imagePathsTmp.OrderBy(path => random2.Next(imagePathsTmp.Count()));"),
        "picker_extensions": require(picker, "SupportFiles.Contains(path.ToUpper().Split('.')[^1])"),
        "picker_zero_image": require(picker, "return null;"),
    }
    for marker in ("time >= 0.75", "time >= 0.5", "time >= 0.25", "lastBmp[i] ??= nextBmp;", "lastBmp[nextScreen] = nextBmp;"):
        require(form,marker)
    for marker in ('"JFIF"', '"JPEG"', '"JPG"', '"PNG"', "Directory.GetFiles(folderPath)", "Bitmap(File.OpenRead("):
        require(picker,marker)
    samples = [
        0, 0.03, 0.15, 0.24999, 0.25, 0.27, 0.49999,
        0.5, 0.51, 0.74999, 0.75, 0.9, 0.99, 1
    ]
    t = f32(0)
    ticks = []
    while t <= 1:
        ticks.append(t)
        t = f32(t + f32(0.03))
        if len(ticks)>50:
            raise AssertionError("unexpected time loop")
    return {
        "schema_version":1,
        "source": {"branch":"master", "commit":REV,
                   "files":{FORM:{"sha256":form_sha},PICKER:{"sha256":picker_sha}},
                   "method":"source extraction; C# executable was not run"},
        "numeric_provenance":{"status":"independent approximation, not C# runtime golden",
                              "method":"Python math.cosh rounded to binary32 with struct.pack/unpack at C#-style intermediate steps",
                              "tolerance_pending":"T031 must compare actual .NET MathF.Cosh and agree explicit numeric tolerance",
                              "samples_are_direct_times":"Some sample times (including exactly 0.25 and 0.5) are branch-edge tests, not values visited by the source's float32 += 0.03 loop."},
        "verbatim_csharp_expressions":expressions,
        "constants":{"normal_slideshow_delay_ms_release":30000,"normal_slideshow_delay_ms_debug":1000,
                     "animation_frame_delay_ms":40,"normal_progress_step_f32":f32(.03),
                     "original_startup_progress_step_f32":f32(1),
                     "original_rrss_f32":f32(.2),"original_rotation_min_inclusive":-20,
                     "original_rotation_max_exclusive":20,
                     "phase_thresholds":[.25,.5,.75],
                     "white_border_total_pixels":20,"white_border_margin_each_side_pixels":10,
                     "original_startup_successful_switches":"monitor_count * 10 + 1 globally (no per-monitor guarantee)"},
        "phase_logic":[
            {"range":"t < 0.25","effect":"old image transform, no new image visible"},
            {"range":"0.25 <= t < 0.5","effect":"new white vertical reveal mask, timeB = 1 / cosh((4t-2)*2pi)"},
            {"range":"0.5 <= t < 0.75","effect":"new image beneath moving white mask, timeB = 1 - 1 / cosh((4t-2)*2pi)"},
            {"range":"t >= 0.75","effect":"new image fully displayed"}],
        "float32_direct_samples":[sample_curve(x) for x in samples],
        "normal_loop_time_inputs_f32":ticks,
        "new_bitmap_geometry_cases":[
            image_geom(3840,2160,1920,1080,0,0),
            image_geom(800,1200,1920,1080,-1920,0),
            image_geom(2000,2000,2560,1440,0,-1440),
            image_geom(1920,1080,1920,1080,-2560,-1080),
            image_geom(4000,1000,1280,1024,1920,0),
        ],
        "monitor_weight_cases":[
            weighted_case([(1920,1080),(1280,1024)],[0,0.001,0.1,0.4,0.61,0.8,0.999999]),
            weighted_case([(1920,1080)],[0,.25,.999999]),
            weighted_case([(1920,1080),(1920,1080),(2560,1440)],[0,.3,.6,.95])],
        "random_position_contract":{
            "initial_x_draw":"Random.Next((int)(screenWidth * 0.4))",
            "if_initial_x_below":"screenWidth * 0.2",
            "then_subtract_f32":"screenWidth * 0.4f",
            "same_for_y":True,
            "rotation":"Random.Next(-20, 20)",
            "not_seeded_here":"Exact System.Random seed/permutation requires matching .NET version; this fixture tests fixed draws, not the RNG implementation."
        },
        "image_picker_original": {
            "global_single_picker":True,
            "extensions_case_insensitive":["JFIF","JPEG","JPG","PNG"],
            "scan_scope":"Directory.GetFiles(folderPath) (top level, nonrecursive)",
            "on_every_next":"Increment index, wrap at path count; inspect each directory's file count and current file existence; refresh only on change, not on cycle rollover alone",
            "refresh_sort":"OrderBy(path => random.Next(imagePathsTmp.Count())) with cached tick seed; key ties permitted; not Fisher-Yates",
            "deduplicates_paths":False,
            "new_cycle_can_repeat_prior_order":True,
            "reset":"Sets index=-1 without scan",
            "single_image":"reused on subsequent index rollovers",
            "zero_image":"returns null after a scan-dependent check",
            "decode_failure_handling":"No internal catch around Bitmap(File.OpenRead(...)); errors may propagate",
            "new_rust_behavior_changes":"approved rescan every completed cycle, fresh unbiased shuffle, deduplicate paths, safe zero/corrupt file handling; do not infer these from master",
        },
        "legacy_monitor_state":{
            "selection":"area-weighted random index from Screen.AllScreens",
            "centers":"screen.Bounds.Left + screen.Bounds.Width / 2, similarly top/height, with integer division",
            "fallback":"Other screens without a lastBmp get nextBmp on first successful pick (lastBmp[i] ??= nextBmp)",
            "commit_on_success":"lastBmp[nextScreen] = nextBmp; others retain previous bitmap",
            "negative_origin_supported_by_center_formula":True,
            "placement_coordinate_caveat":"C# HWndGraphics/virtual desktop origin differences must be verified on Windows; do not assert pixel-perfect negative-origin rendering yet."
        }
    }


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--write",action="store_true",help="explicitly generate frozen fixture")
    args=parser.parse_args()
    expected=json.dumps(make(),ensure_ascii=False,indent=2,sort_keys=True)+"\n"
    if args.write:
        TARGET.write_text(expected,encoding="utf-8")
        print("WROTE",TARGET,len(expected.encode("utf-8")))
    else:
        if TARGET.read_text(encoding="utf-8") != expected:
            raise SystemExit("FAIL: fixture differs from independently regenerated source-derived vector set")
        print("PASS: immutable fixture matches pinned master and deterministic binary32 approximation generator")


if __name__=="__main__":
    main()
