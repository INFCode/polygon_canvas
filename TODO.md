# Milestone to v0.1.0

## Canvas

- [ ] Rename Canvas to ImageArray, CanvasSpec into ImageShape
- [ ] Stop using Array2, and replace it with ImageBuffer or its alias
- [ ] Load from image
- [ ] Save to image
- [ ] Move some functionalities (size, dimensions) to ImageShape
- [ ] Initialize as a fully white image

## Algorithms

### Fill-Polygon

- [ ] Fix overlapping boundary rendering
- [ ] Use correct blending mode (at least from burn to multiply)

### Similarity

- [ ] Define ImageSimilarity trait
- [ ] Support MSE

## Engine

- [ ] Add reference image and image similarity metric to Engine
- [ ] Add a step function for moving forward and giving similarity feedback

# Future TODO

## ImageArray

- [ ] Support serializing the ImageArray. Maybe with compression, but must be lossless

## Algorithm

### Similarity


## Geometry

- [ ] Support serializing the diffenet geometric objects

## Algorithm

### Similarity

- [ ] Support preceptrual hash
- [ ] Support PSNR
- [ ] Support SSIM & MS-SSIM
- [ ] Support histogram simialrity

### Fill-Polygon

- [ ] Profile it

## Interface

- [ ] Provide a python interface throuhg pyo3

## Performance

- [ ] Parallelize some computation
