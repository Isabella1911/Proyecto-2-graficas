# Proyecto-2-graficas
Video: https://youtu.be/4B7ErZqEG9A

Características principales implementadas

Piso con capas de dirt + grass

Casa completa 

Árbol 

Lago / agua

Sol físico en el cielo

Materiales con texturas reales en carpeta assets/

Todos los bloques se generan con builder.rs.

2. Cielo procedural con ciclo día/noche suave

Usa un módulo DayNight con:

Color de cielo dependiente de la hora

Sol que se mueve en el cielo con dirección realista

Color del sol según elevación (amanecer → dorado → mediodía → atardecer)

Intensidad solar suave y progresiva

Iluminación difusa + término ambiental ajustado a la hora

El cielo no usa texturas externas, todo es procedural.

3. Materiales con texturas externas

Los materiales cargan imágenes desde assets/textures/, por ejemplo:

grass.jpeg

dirt.jpeg

stone.jpeg

planks.jpeg

glass.jpeg

water.png

tree.jpeg


4. Framebuffer flotante dibujado con Raylib

Exportar frames para generar video


5. Multi-threading para aumentar FPS

La renderización por tiles se divide entre varios threads usando rayon o spawn_threads.
Esto permite mantener 30–60 FPS, dependiendo de resolución y SPP.

6. Cámara controlable por el usuario

La cámara ahora se mueve con el teclado:

Movimiento horizontal:

W → adelante

S → atrás

A → izquierda

D → derecha

Movimiento vertical:

UP → subir

DOWN → bajar

Órbita opcional:

La cámara también puede orbitar lentamente por tiempo.
