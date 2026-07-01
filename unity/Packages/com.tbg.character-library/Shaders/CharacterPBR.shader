Shader "TBG/CharacterPBR"
{
    Properties
    {
        _MainTex ("Albedo", 2D) = "white" {}
        _Color ("Tint", Color) = (1,1,1,1)
        _BumpMap ("Normal", 2D) = "bump" {}
        _ORMMap ("ORM (R=AO G=Rough B=Metal)", 2D) = "white" {}
        _Metallic ("Metallic Multiplier", Range(0,1)) = 0.1
        _Smoothness ("Smoothness Multiplier", Range(0,1)) = 0.5
        _OcclusionStrength ("Occlusion Strength", Range(0,1)) = 1
        _EmissionMap ("Emissive", 2D) = "black" {}
        _EmissionStrength ("Emission Strength", Range(0,2)) = 0
    }
    SubShader
    {
        Tags { "RenderType"="Opaque" }
        LOD 200

        CGPROGRAM
        #pragma surface surf Standard fullforwardshadows
        #pragma target 3.0

        sampler2D _MainTex;
        sampler2D _BumpMap;
        sampler2D _ORMMap;
        sampler2D _EmissionMap;
        fixed4 _Color;
        half _Metallic;
        half _Smoothness;
        half _OcclusionStrength;
        half _EmissionStrength;

        struct Input
        {
            float2 uv_MainTex;
        };

        void surf(Input IN, inout SurfaceOutputStandard o)
        {
            fixed4 albedo = tex2D(_MainTex, IN.uv_MainTex) * _Color;
            fixed3 orm = tex2D(_ORMMap, IN.uv_MainTex).rgb;
            fixed ao = lerp(1.0h, orm.r, _OcclusionStrength);
            half roughness = orm.g;
            half metallic = orm.b * _Metallic;
            half smoothness = (1.0h - roughness) * _Smoothness;

            o.Albedo = albedo.rgb * ao;
            o.Normal = UnpackNormal(tex2D(_BumpMap, IN.uv_MainTex));
            o.Metallic = metallic;
            o.Smoothness = smoothness;
            o.Occlusion = ao;
            o.Emission = tex2D(_EmissionMap, IN.uv_MainTex).rgb * _EmissionStrength;
        }
        ENDCG
    }
    FallBack "Diffuse"
}
