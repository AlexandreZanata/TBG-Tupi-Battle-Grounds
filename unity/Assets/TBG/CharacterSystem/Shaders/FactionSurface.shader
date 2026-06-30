Shader "TBG/FactionSurface"
{
    Properties
    {
        _PrimaryColor ("Primary", Color) = (1,1,1,1)
        _SecondaryColor ("Secondary", Color) = (0.8,0.8,0.8,1)
        _FactionMode ("Faction", Float) = 0
        _EvolutionGlow ("Evolution Glow", Range(0,1)) = 0
    }
    SubShader
    {
        Tags { "RenderType"="Opaque" }
        LOD 200

        CGPROGRAM
        #pragma surface surf Standard fullforwardshadows
        #pragma target 3.0

        struct Input
        {
            float3 worldPos;
            float2 uv;
        };

        fixed4 _PrimaryColor;
        fixed4 _SecondaryColor;
        float _FactionMode;
        float _EvolutionGlow;

        float hash(float2 p)
        {
            return frac(sin(dot(p, float2(127.1, 311.7))) * 43758.5453);
        }

        void surf(Input IN, inout SurfaceOutputStandard o)
        {
            float noise = hash(IN.worldPos.xz * 4.0);
            float pattern = lerp(noise, sin(IN.worldPos.y * 10.0), step(1.5, _FactionMode));
            float metallic = lerp(0.1, 0.85, step(1.5, _FactionMode) * step(_FactionMode, 2.5));
            float emissive = _EvolutionGlow * 0.35;

            if (_FactionMode > 2.5)
            {
                emissive += 0.15 * (0.5 + 0.5 * sin(_Time.y * 2.0 + IN.worldPos.x));
            }

            fixed4 baseColor = lerp(_PrimaryColor, _SecondaryColor, pattern);
            o.Albedo = baseColor.rgb;
            o.Metallic = metallic;
            o.Smoothness = lerp(0.35, 0.75, metallic);
            o.Emission = baseColor.rgb * emissive;
        }
        ENDCG
    }
    FallBack "Diffuse"
}
