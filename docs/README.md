# Proyecto-2-graficas

---
**Video**: [Ver video en YouTube](https://youtu.be/ymqGnhap4e4)
## Características principales

- **Escena voxel**:
  - Terreno de dirt/grass.
  - Casa con paredes delgadas, techo escalonado, puerta y ventanas.
  - Árbol tipo Minecraft (tronco + copa por capas).
  - Charco de agua.
  - Poste con antorcha frente a la casa.


-  **Iluminación y materiales**:
  - Materiales con parámetros de:
    - `albedo`
    - `specular`
    - `emissive`
    - `reflection`
  - Materiales distintos para:
    - Grass, dirt, stone, planks, dark_wood, roof, glass, water, torch, tree_leaves, sun.
  - **Antorchas emisivas** que iluminan con rayos de sombra.
  - **Sol físico** como bloque emisivo en el cielo.

- **Ciclo día/noche**:
  - Sistema `DayNight` que:
    - Calcula la dirección del sol según el tiempo.
    - Ajusta la intensidad y color del sol.
    - Calcula el color del cielo (soft summer) y la luz ambiental.
  - El cielo se actualiza frame a frame para simular el paso del tiempo.

-  **Animación y cámara**:
  - Cámara orbital (`CameraOrbit`) alrededor de la casa:
    - Rotación suave.
    - Zoom in/out (radio variable).
  - Se genera un **timelapse**:
    - `fps`, duración y número de frames configurables en `main.rs`.

- **Multithreading**:
  - El framebuffer se divide en tiles.
  - Cada tile se renderiza en un hilo separado.
  - Los resultados se combinan al final de cada frame.

-  **Texturas**:
  - Carga de texturas desde `assets/textures/`:
    - `grass.jpeg`, `dirt.jpeg`, `stone.jpeg`, `planks.jpeg`,
      `roof.jpeg`, `glass.jpeg`, `tree.jpeg`, `water.png`.
  

---
## Para correr
cargo run --release


guarda frame_XXXX.bmp/ png
