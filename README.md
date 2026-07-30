# auto-doro

This project was built by GPT-OSS 120B and Gemini 3.1 Pro High/Low.

## Overview

This project is a lightweight 3D model viewer built in Rust using the Bevy game engine. It demonstrates how to create a borderless, transparent multi-window application. The primary window displays an interactive 3D model (specifically a McLaren 720S) with a dynamic orbit camera that slowly rotates around the model.

Users can interact with the primary window by clicking and dragging near the edges to move the window across the screen, or by clicking and dragging in the center to manually rotate the camera. Scrolling the mouse wheel zooms the camera in and out. A secondary, synchronized window built with bevy_egui floats alongside the main viewer to display a sleek, semi-transparent UI with car specifications.
