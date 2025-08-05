#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Created on Fri Jan 31 23:03:55 2025

@author: halt
"""

import os
import sys

import plotly.graph_objects as go
import plotly.offline as py
import shapely.geometry as geo
import svgelements
from devtools import debug
from svgelements import SVG


def main(av: list[str]) -> int:
    if len(av) != 2:
        print("usage: lanyard.py SHAPE.svg", file=sys.stderr)
        return os.EX_USAGE

    svg = SVG.parse(av[1])

    for element in svg.elements():
        debug(element)

        if isinstance(element, list):
            for subelement in element:
                if isinstance(subelement, svgelements.svgelements.Polygon):
                    debug(subelement.points)
                    points = list(map(lambda p: (p.x, p.y), subelement.points))
                    poly = geo.Polygon(points)
                    x, y = poly.exterior.xy
                    debug(poly.centroid)
                    centroid_x, centroid_y = poly.centroid.x, poly.centroid.y
                    distances = []

                    for point in points:
                        distance = poly.centroid.distance(geo.Point(point))
                        distances.append(distance)
                        debug(point, distance)

                    trace = [
                        go.Scatter(x=list(x), y=list(y), mode="lines+markers", name="polygon"),
                        go.Scatter(
                            x=[centroid_x],
                            y=[centroid_y],
                            mode="markers",
                            name="centroid",
                        ),
                    ]
                    fig = go.Figure(data=trace)

                    for distance in distances:
                        fig.add_shape(
                            type="circle",
                            xref="x",
                            yref="y",
                            x0=centroid_x - distance,
                            y0=centroid_y - distance,
                            x1=centroid_x + distance,
                            y1=centroid_y + distance,
                            line_color="red",
                            opacity=0.2,
                        )

                    py.plot(fig)
                elif isinstance(subelement, svgelements.svgelements.Polyline):
                    ...

    return os.EX_OK


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except KeyboardInterrupt:
        sys.exit(os.EX_OK)
