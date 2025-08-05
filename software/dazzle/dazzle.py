#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Created on Fri Apr 11 23:03:55 2025

@author: halt
"""

import logging
import os
import random
import sys
from collections.abc import Callable, Iterator
from os import PathLike
from typing import Any, Final

import matplotlib.pyplot as plt
import numpy as np
import shapely
import shapely.affinity
import shapely.geometry as geo
import shapely.plotting
import svgelements
from devtools import debug
from shapely.affinity import rotate, translate
from shapely.ops import unary_union
from sklearn.mixture import GaussianMixture
from svgelements import SVG

X_OFFSET: Final[float] = 53.6996
X_SCALE: Final[float] = 2.7911630313495452
Y_OFFSET: Final[float] = 146.0934
Y_SCALE: Final[float] = 2.712167801821898

# NOTE: Actual dimensions are 5mm x 5mm
WS2812B_LENGTH: Final[float] = 5.0 * X_SCALE
WS2812B_WIDTH: Final[float] = 5.0 * Y_SCALE


def create_rectangle_from_point(
    top_left: geo.Point, length: float = WS2812B_LENGTH, width: float = WS2812B_WIDTH
) -> geo.Polygon:
    coords = (
        top_left,
        (top_left.x + width, top_left.y),
        (top_left.x + width, top_left.y - length),
        (top_left.x, top_left.y - length),
    )
    square = geo.Polygon(coords)
    return square


LEDS: tuple[geo.Polygon, ...] = tuple(
    map(
        lambda p: create_rectangle_from_point(
            geo.Point(((p[0] - X_OFFSET) * X_SCALE, Y_SCALE * (Y_OFFSET - p[1])))
        ),
        (
            (54.7, 122.1),
            (99.2, 114.6),
            (143.7, 106.6),
            (187.7, 98.6),
            (232.2, 91.1),
            (99.7, 139.1),
            (134.7, 139.1),
            (169.7, 139.1),
            (204.7, 139.1),
        ),
    )
)

_led_centroid = unary_union(LEDS).centroid
LEDS = tuple([rotate(p, angle=-10, origin=_led_centroid) for p in LEDS])
LEDS = tuple([translate(p, xoff=10, yoff=30) for p in LEDS])

# LOGO_WIDTH: Final[float] = 75.0
# LOGO_LENGTH: Final[float] = 40.0
# LOGO: Final[geo.Polygon] = create_rectangle_from_point(geo.Point(278.319 - LOGO_WIDTH / 2, 60.667 + LOGO_LENGTH / 2), LOGO_LENGTH, LOGO_WIDTH)

_log_handler = logging.StreamHandler(sys.stdout)
_log_formatter = logging.Formatter(
    "[%(processName)s][%(threadName)s] %(asctime)s - %(name)s:%(lineno)d - %(levelname)s - %(message)s"
)
_log_handler.setFormatter(_log_formatter)
logger = logging.getLogger()
logger.addHandler(_log_handler)
logger.setLevel(logging.DEBUG)


def linestring_to_polygon(line: geo.LineString) -> geo.Polygon:
    if not line.is_ring:
        closed_coords = list(line.coords) + [line.coords[0]]
    else:
        closed_coords = list(line.coords)

    return geo.Polygon(closed_coords)


def svg_to_polygons(svg_file: PathLike | str) -> Iterator[geo.Polygon]:
    svg = SVG.parse(svg_file)
    found = False

    for element in svg.elements():
        if isinstance(element, list):
            for subelement in element:
                if isinstance(subelement, svgelements.svgelements.Polygon):
                    points = list(map(lambda p: (p.x, p.y), subelement.points))
                    poly = geo.Polygon(points)
                    yield poly
                    found = True
                elif isinstance(subelement, svgelements.svgelements.Polyline):
                    debug(subelement)
                    line = geo.LineString(list(map(lambda p: (p.x, p.y), subelement.points)))
                    poly = linestring_to_polygon(line)
                    yield poly
                    found = True
                else:
                    logger.debug(f"SubElement[{type(element)}: {element}")
        else:
            logger.debug(f"Element[{type(element)}: {element}")

    if not found:
        raise RuntimeError(f"No polygons found in {svg_file}!")


def svg_to_polygon(svg_file: PathLike | str) -> geo.Polygon:
    return next(svg_to_polygons(svg_file))


def normalize_polygon(
    poly: geo.Polygon,
    origin: geo.Point | str | None = None,
    rotate: bool = True,
    flip: bool = True,
    min_x: float | None = None,
    min_y: float | None = None,
) -> tuple[geo.Polygon, float, float]:
    if origin is None:
        origin = "centroid"

    if rotate:
        poly = shapely.affinity.rotate(poly, 180, origin=origin)

    if flip:
        poly = shapely.affinity.scale(poly, xfact=-1, yfact=1, origin=origin)

    _min_x, _min_y, _, _ = poly.bounds

    if min_x is None:
        min_x = _min_x

    if min_y is None:
        min_y = _min_y

    poly = shapely.affinity.translate(poly, xoff=-min_x, yoff=-min_y)

    return poly, min_x, min_y


def sample_point(
    poly: geo.Polygon,
    max_iterations: int = 100,
) -> geo.Point:
    min_x, min_y, max_x, max_y = poly.bounds
    i = 0

    while True:
        point = geo.Point(random.uniform(min_x, max_x), random.uniform(min_y, max_y))

        if poly.contains(point):
            return point

        i += 1

        if i >= max_iterations:
            raise RuntimeError(f"Maximum iterations exceeded sampling for a point: {i}")


def sample_rectangle(
    poly: geo.Polygon,
    length: float = WS2812B_LENGTH,
    width: float = WS2812B_WIDTH,
    max_iterations: int = 100,
) -> geo.Polygon:
    i = 0

    while True:
        top_left = sample_point(poly)
        coords = (
            top_left,
            (top_left.x + width, top_left.y),
            (top_left.x + width, top_left.y - length),
            (top_left.x, top_left.y - length),
        )
        square = geo.Polygon(coords)

        if poly.contains(square):
            return square

        i += 1

        if i >= max_iterations:
            raise RuntimeError(f"Maximum iterations exceeded sampling for a square: {i}")


def check_rectangle_overlap(rectangles: list[geo.Polygon]) -> None:
    for i in range(len(rectangles)):
        for j in range(1, len(rectangles)):
            if i == j:
                continue

            r0 = rectangles[i]
            r1 = rectangles[j]

            if r0.intersects(r1):
                raise RuntimeError(f"Rectangles {r0} and {r1} intersect!")


def rectangles_to_multipoint(rectangles: list[geo.Polygon]) -> geo.MultiPoint:
    points = []

    for rectangle in rectangles:
        point = rectangle.centroid
        points.append(point)

    return geo.MultiPoint(points)


def extract_elements_from_collection(collection: Any) -> list[geo.Polygon]:
    polys = []

    if isinstance(collection, geo.GeometryCollection):
        for geom in collection.geoms:
            polys.extend(extract_elements_from_collection(geom))
    elif isinstance(collection, geo.Polygon):
        return [collection]
    # elif isinstance(collection, geo.MultiLineString):
    #     return [collection]
    else:
        raise NotImplementedError(
            f"Collection extraction not supported for {type(collection)}... yet!"
        )

    return polys


def rectangle_from_polygon(poly: geo.Polygon) -> geo.Polygon:
    rectangle = poly.oriented_envelope.normalize()

    if not isinstance(rectangle, geo.Polygon):
        raise RuntimeError("Cannot extract oriented envelope: %s => %s", poly, rectangle)

    return rectangle


def plot_rectangle_from_polygon(poly: geo.Polygon, ax: Any) -> None:
    rectangle = rectangle_from_polygon(poly)
    shapely.plotting.plot_polygon(
        rectangle,
        color="none",
        facecolor="white",
        edgecolor="none",
        ax=ax,
    )


Color = str | tuple[float, float, float] | tuple[float, float, float, float]


def wang_from_polygon(
    poly: geo.Polygon,
) -> list[tuple[geo.Polygon, Color]]:
    rectangle = rectangle_from_polygon(poly)
    centroid = rectangle.centroid
    corners = list(rectangle.exterior.coords)[:-1]
    triangles = []

    for i in range(len(corners)):
        p1 = corners[i]
        p2 = corners[(i + 1) % len(corners)]
        triangle = geo.Polygon([centroid.coords[0], p1, p2])
        triangles.append((triangle, random.choice(["black", "white"])))

    return triangles


def truchet_from_polygon(poly: geo.Polygon) -> list[tuple[geo.Polygon, Color]]:
    rectangle = rectangle_from_polygon(poly)
    corners = list(rectangle.exterior.coords)[:-1]
    top_left, top_right, bottom_right, bottom_left = corners
    slice_a1 = geo.Polygon((top_left, bottom_left, bottom_right))
    slice_a2 = geo.Polygon((top_right, bottom_right, bottom_left))
    slice_b1 = geo.Polygon((top_left, top_right, bottom_right))
    slice_b2 = geo.Polygon((top_left, bottom_left, bottom_right))

    match random.choice([1, 2, 3, 4]):
        case 1:
            return [(slice_a1, "white"), (slice_a2, "black")]
        case 2:
            return [(slice_b1, "black"), (slice_b2, "white")]
        case 3:
            return [(slice_a1, "black"), (slice_a2, "white")]
        case 4:
            return [(slice_b1, "white"), (slice_b2, "black")]
        case _:
            assert False, "unreachable"


def black_or_white_from_polygon(poly: geo.Polygon) -> list[tuple[geo.Polygon, Color]]:
    return [(rectangle_from_polygon(poly), random.choice(["white", "black"]))]


def coerce_to_polygon(geom: Any) -> geo.Polygon:
    if isinstance(geom, geo.Polygon):
        return geom
    elif isinstance(geom, geo.LineString):
        coords = list(geom.coords)

        if geom.is_ring:
            return geo.Polygon(coords)

        return geo.Polygon(coords + [coords[0]])
    else:
        # TODO Failed seeds: 829902259
        raise NotImplementedError(f"Polygon coercion not supported for {type(geom)}... yet!")


def plot_tiling_from_polygon(
    envelope: geo.Polygon,
    white_constraints: list[geo.Polygon],
    black_constraints: list[geo.Polygon],
    poly: geo.Polygon,
    tiling: Callable[[geo.Polygon], list[tuple[geo.Polygon, Color]]],
    ax: Any,
) -> None:
    tiles = tiling(poly)

    for tile, color in tiles:
        try:
            tile = coerce_to_polygon(tile.intersection(envelope).intersection(poly))
        except NotImplementedError as error:
            logger.warning(error)
            continue

        # Black constraints first so the LEDs don't get overwritten.
        for constraint in black_constraints:
            if tile.intersects(constraint) or tile.contains(constraint):
                color = "black"

        for constraint in white_constraints:
            if tile.intersects(constraint) or tile.contains(constraint):
                color = "white"

        shapely.plotting.plot_polygon(
            tile,
            color="none",
            facecolor=color,
            edgecolor="none",
            ax=ax,
        )


def generate_gmm(
    covariance_norms: list[float], envelope: geo.Polygon, seed: int | None = None
) -> GaussianMixture:
    debug(covariance_norms)
    rng = np.random.default_rng(seed)
    num_components = len(covariance_norms)
    min_x, min_y, max_x, max_y = envelope.bounds
    means = np.column_stack(
        (
            rng.uniform(min_x, max_x, size=num_components),
            rng.uniform(min_y, max_y, size=num_components),
        )
    )

    # NOTE: Screw around with this scalar to make the points more or less dense.
    covariance_scalar = envelope.area
    covariance_scales = list(map(lambda c: covariance_scalar**c, covariance_norms))
    covariances = np.array(
        [np.eye(2) * np.array([covariance_scale]) for covariance_scale in covariance_scales]
    )
    weights = rng.uniform(0, 1, size=num_components)
    weights /= np.sum(weights)
    n_samples = 1000
    samples = []
    component_choices = rng.choice(num_components, size=n_samples, p=weights)

    for comp in component_choices:
        sample = rng.multivariate_normal(means[comp], covariances[comp])
        samples.append(sample)

    gmm = GaussianMixture(n_components=num_components, covariance_type="full", random_state=seed)
    X = np.array(samples)
    gmm.fit(X)

    return gmm


def sample_gmm(gmm: GaussianMixture, num_samples: int) -> list[geo.Point]:
    if num_samples == 0:
        return []

    samples = gmm.sample(num_samples)[0]
    points = []

    for sample in samples:
        points.append(geo.Point((sample[0], sample[1])))

    return points


def sample_gmm_in_envelope(
    gmm: GaussianMixture, envelope: geo.Polygon, num_samples: int
) -> list[geo.Point]:
    if num_samples == 0:
        return []

    samples = []

    while len(samples) < num_samples:
        debug(len(samples))
        new_samples = sample_gmm(gmm, num_samples)

        for sample in new_samples:
            if envelope.contains(sample):
                samples.append(sample)

                if len(samples) >= num_samples:
                    return samples

    return samples


def generate_dazzle(
    seed: int,
    envelope: geo.Polygon,
    contour_white: list[geo.Polygon],
    contour_black: list[geo.Polygon],
    num_clusters: int,
    num_points: int,
    cluster_led_centroids: bool = False,
    cluster_envelope: bool = False,
) -> None:
    logger.debug("Dazzle seed: %s", seed)
    random.seed(seed)
    fig, axs = plt.subplots(4, 3)
    skips = (axs[0][0], axs[2][0])

    for row in axs:
        for ax in row:
            ax.axis("off")

            # Allow the point clusters to auto-scale.
            if ax in skips:
                continue

            min_x, min_y, max_x, max_y = envelope.bounds
            width = max_x - min_x
            height = max_y - min_y
            ax.set_xlim(min_x - width * 0.1, max_x + width * 0.1)
            ax.set_ylim(min_y - height * 0.2, max_y + height * 0.2)

    gmm = generate_gmm(
        list(map(lambda _: random.random(), range(num_clusters))), envelope, seed=seed
    )
    control_points = sample_gmm(gmm, num_points)
    control_multipoint = geo.MultiPoint(control_points)
    shapely.plotting.plot_points(control_multipoint, color="xkcd:dried blood", ax=axs[0][0])
    shapely.plotting.plot_points(control_multipoint, color="xkcd:dried blood", ax=axs[2][0])
    leds = list(LEDS)
    # check_rectangle_overlap(leds + [LOGO])

    if cluster_led_centroids:
        led_centroids = rectangles_to_multipoint(leds)
        contour_centroids = rectangles_to_multipoint(contour_white).union(
            rectangles_to_multipoint(contour_black)
        )
        voronoi_multipoint = contour_centroids.union(led_centroids.union(control_multipoint))
    else:
        voronoi_multipoint = control_multipoint

    if cluster_envelope:
        voronoi_multipoint = voronoi_multipoint.union(envelope.boundary)

    voronoi = shapely.voronoi_polygons(voronoi_multipoint).normalize()
    voronoi_elems = extract_elements_from_collection(voronoi)
    random.shuffle(voronoi_elems)
    voronoi_led_elems = []
    voronoi_logo_elems = []

    for voronoi_elem in voronoi_elems:
        for led in leds:
            if voronoi_elem.contains(led):
                assert voronoi_elem.contains_properly(
                    led
                ), "LED must be fully contained in a Voronoi cell."
                voronoi_led_elems.append(voronoi_elem)

        for constraint in contour_white + contour_black:
            if voronoi_elem.contains(constraint):
                assert voronoi_elem.contains_properly(
                    constraint
                ), "Contour must be fully contained in a Voronoi cell."
                voronoi_logo_elems.append(voronoi_elem)

    for voronoi_elem in voronoi_elems:
        try:
            voronoi_elem = coerce_to_polygon(voronoi_elem.intersection(envelope))
        except NotImplementedError as error:
            logger.warning(error)
            continue

        if voronoi_elem.is_empty:
            continue

        try:
            voronoi_rectangle = rectangle_from_polygon(voronoi_elem)
        except RuntimeError as error:
            logger.warning(error)
            continue

        shapely.plotting.plot_polygon(
            voronoi_elem, color="none", facecolor="none", edgecolor="xkcd:pale red", ax=axs[0][1]
        )
        shapely.plotting.plot_polygon(
            voronoi_rectangle,
            color="none",
            facecolor="none",
            edgecolor="xkcd:pale red",
            ax=axs[0][2],
        )
        plot_tiling_from_polygon(
            envelope,
            leds + contour_white,
            contour_black,
            voronoi_elem,
            wang_from_polygon,
            axs[1][0],
        )
        plot_tiling_from_polygon(
            envelope,
            leds + contour_white,
            contour_black,
            voronoi_elem,
            truchet_from_polygon,
            axs[1][1],
        )
        plot_tiling_from_polygon(
            envelope,
            leds + contour_white,
            contour_black,
            voronoi_elem,
            black_or_white_from_polygon,
            axs[1][2],
        )

    delaunay_multipoint = voronoi_multipoint
    delaunay = shapely.delaunay_triangles(delaunay_multipoint).normalize()
    delaunay_elems = extract_elements_from_collection(delaunay)
    random.shuffle(delaunay_elems)
    delaunay_led_elems = []
    delaunay_logo_elems = []

    for delaunay_elem in delaunay_elems:
        for led in leds:
            if delaunay_elem.contains(led):
                assert delaunay_elem.contains_properly(
                    led
                ), "LED must be fully contained in a Delaunay cell."
                delaunay_led_elems.append(delaunay_elem)

        for constraint in contour_white + contour_black:
            if delaunay_elem.contains(constraint):
                assert delaunay_elem.contains_properly(
                    constraint
                ), "Contour must be fully contained in a Delaunay cell."
                delaunay_logo_elems.append(delaunay_elem)

    for delaunay_elem in delaunay_elems:
        try:
            delaunay_elem = coerce_to_polygon(delaunay_elem.intersection(envelope))
        except NotImplementedError as error:
            logger.warning(error)
            continue

        if delaunay_elem.is_empty:
            continue

        try:
            delaunay_rectangle = rectangle_from_polygon(delaunay_elem)
        except RuntimeError as error:
            logger.warning(error)
            continue

        shapely.plotting.plot_polygon(
            delaunay_elem, color="none", facecolor="none", edgecolor="xkcd:sky blue", ax=axs[2][1]
        )
        shapely.plotting.plot_polygon(
            delaunay_rectangle,
            color="none",
            facecolor="none",
            edgecolor="xkcd:sky blue",
            ax=axs[2][2],
        )
        plot_tiling_from_polygon(
            envelope,
            leds + contour_white,
            contour_black,
            delaunay_elem,
            wang_from_polygon,
            axs[3][0],
        )
        plot_tiling_from_polygon(
            envelope,
            leds + contour_white,
            contour_black,
            delaunay_elem,
            truchet_from_polygon,
            axs[3][1],
        )
        plot_tiling_from_polygon(
            envelope,
            leds + contour_white,
            contour_black,
            delaunay_elem,
            black_or_white_from_polygon,
            axs[3][2],
        )

    for row in axs:
        for ax in row:
            shapely.plotting.plot_polygon(
                envelope,
                color="none",
                facecolor="none",
                edgecolor="xkcd:neon pink",
                linewidth=2.0,
                ax=ax,
            )

    if os.getenv("PLOT_CONSTRAINTS", "false").lower() == "true":
        for constraint in contour_white:
            for row in axs:
                for ax in row:
                    shapely.plotting.plot_polygon(
                        constraint,
                        color="none",
                        facecolor="xkcd:lavender",
                        edgecolor="xkcd:neon pink",
                        ax=ax,
                    )

        for constraint in contour_black:
            for row in axs:
                for ax in row:
                    shapely.plotting.plot_polygon(
                        constraint,
                        color="none",
                        facecolor="xkcd:salmon",
                        edgecolor="xkcd:neon pink",
                        ax=ax,
                    )

        for led in leds:
            for row in axs:
                for ax in row:
                    shapely.plotting.plot_polygon(
                        led,
                        color="none",
                        facecolor="xkcd:rose pink",
                        edgecolor="xkcd:neon pink",
                        ax=ax,
                    )

    plt.grid(False)
    width_px = 3840
    height_px = 2160
    dpi = 100
    width_in = width_px / dpi
    height_in = height_px / dpi

    fig.set_size_inches(width_in, height_in)
    plt.savefig(
        f"{num_clusters}_{num_points}_{'led' if cluster_led_centroids else 'noled'}_{'envelope' if cluster_envelope else 'noenvelope'}_{seed}.svg",
        dpi=dpi,
    )


def main(av: list[str]) -> int:
    if len(av) not in (2, 3):
        print("usage: dazzle.py SHAPE.svg [SEED]", file=sys.stderr)
        return os.EX_USAGE

    match len(av):
        case 2:
            seed = random.randint(0, 2**32 - 1)
        case 3:
            seed = int(av[2])
        case _:
            assert False, "unreachable"

    logger.info("Loading envelope polygon from SVG: %s", av[1])

    template = list(svg_to_polygons(av[1]))
    debug(template)
    # Assume that the first polygon is our border. Not a robust assumption lol. Check your SVG file if this acts up.
    union = unary_union(template)
    rotation_origin = union.centroid
    debug(rotation_origin)
    # envelope = template[0]
    envelope, min_x, min_y = normalize_polygon(template[0], origin=rotation_origin)
    debug(envelope)
    # contour = template[1:]
    contour = list(
        map(
            lambda t: normalize_polygon(
                t, origin=rotation_origin, rotate=True, flip=True, min_x=min_x, min_y=min_y
            )[0],
            template[1:],
        )
    )
    debug(contour)
    contour_white = []
    contour_black = []

    for constraint in contour:
        if random.choice([True, False]):
            contour_white.append(constraint)
        else:
            contour_black.append(constraint)

    debug(contour_white)
    debug(contour_black)
    logger.info("Setting RNG seed to %s", seed)
    debug(seed)
    random.seed(seed)

    for i in range(10, 21):
        for j in range(100, 1001, 50):
            while True:
                logger.info("Using %s GMM clusters.", i)
                logger.info("Using %s control points.", j)
                subseed = random.randint(0, 2**32)

                try:
                    generate_dazzle(
                        subseed, envelope, contour_white, contour_black, i, j, False, False
                    )
                    generate_dazzle(
                        subseed, envelope, contour_white, contour_black, i, j, False, True
                    )
                    generate_dazzle(
                        subseed, envelope, contour_white, contour_black, i, j, True, False
                    )
                    generate_dazzle(
                        subseed, envelope, contour_white, contour_black, i, j, True, True
                    )
                except Exception as exc:
                    logger.error("Failed to generate dazzle for subseed %s: %s", subseed, exc)
                    continue

                break

    return os.EX_OK


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except KeyboardInterrupt:
        plt.close("all")
        sys.exit(os.EX_OK)
