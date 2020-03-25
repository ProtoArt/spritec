#version 150

in vec2 v_texcoords;
out vec4 color;

uniform sampler2D tex;
uniform sampler2D depth_tex;
uniform float near_plane;
uniform float far_plane;
uniform float tolerance;

// GX:
// -1 0 1
// -2 0 2
// -1 0 1

// GY:
// 1 2 1
// 0 0 0
// -1 -2 -1


void main() {
  color = texelFetch(tex, ivec2(gl_FragCoord), 0);
  vec4 depth = texelFetch(depth_tex, ivec2(gl_FragCoord), 0);

  //Only compute contours for non-background pixels
  if(depth.z != 1)
  {
    // Sobel Pixel Multipliers. Ommiting center pixel value because it will always have a weight of 0
    int multipliers_x[6] = int[6](-1,1,-2,2,-1,1);
    int multipliers_y[6] = int[6](1,2,1,-1,-2,-1);

    // Texel offsets from current pixel to perform kernel multiplication
    ivec2 i_offsets[8] = ivec2[](
         ivec2(-1, 1), // top-left
         ivec2(0,1), // top-center
         ivec2(1,1), // top-right
         ivec2(-1,0),   // center-left
         ivec2(1,0),   // center-right
         ivec2(-1,-1), // bottom-left
         ivec2(0,-1), // bottom-center
         ivec2(1,-1)  // bottom-right
     );

   int index_x[6] = int[6](0,2,3,4,5,7);
   int index_y[6] = int[6](0,1,2,5,6,7);

   float edge_sum_x = 0.0;
   float edge_sum_y = 0.0;

   for (int i = 0; i < 6; i++)
   {
       ivec2 offset_x = i_offsets[index_x[i]];
       ivec2 offset_y = i_offsets[index_y[i]];

       vec4 tex_x_depth = texelFetch(depth_tex, ivec2(gl_FragCoord) + offset_x,0);
       vec4 tex_y_depth = texelFetch(depth_tex, ivec2(gl_FragCoord) + offset_y,0);

       float x_z = (1/(tex_x_depth.z - ((far_plane + near_plane)/(far_plane-near_plane))))*((-2.0 * far_plane * near_plane)/(far_plane-near_plane));
       float y_z = (1/(tex_y_depth.z - ((far_plane + near_plane)/(far_plane-near_plane))))*((-2.0 * far_plane * near_plane)/(far_plane-near_plane));
       edge_sum_x += x_z * multipliers_x[i];
       edge_sum_y += y_z * multipliers_y[i];
   }

   // intensity magnitude = sqrt(Gx^2 + Gy^2)
   float magnitude = sqrt(edge_sum_x * edge_sum_x + edge_sum_y * edge_sum_y);
    // 4.3 is good for bigboi
    // 8.3 worked well for big girl (512x512)
    if(magnitude >= tolerance)
    {
      vec3 darker = vec3(color);
      darker = darker * 0.3;
      color = vec4(darker, 1.0);
    } else {
      //TODO: Get inner edges working from normal information
        // vec3 normal = texelFetch(normal_tex, ivec2(gl_FragCoord), 0).xyz;
        // for(int i = 0; i < 9; i++)
        // {
        //   vec3 neighbour_normal = texelFetch(normal_tex, ivec2(gl_FragCoord) + i_offsets[i], 0).xyz;
        //   float dot_prod = dot(normal, neighbour_normal);
        //   // When normal.z = cs_pos, dot products range from 0 to 1.96 ish
        //   // When normal.z is as usual, dot products range from 0 to 1
        //   if(dot_prod < 0.0)
        //   {
        //     color = vec4(0.0,1.0,0.0,1.0);
        //     break;
        //   }
        // }
      }
    }


}
