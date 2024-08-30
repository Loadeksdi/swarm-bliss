<script lang="ts">
    import Slider from '$lib/Slider.svelte';
    import { invoke } from '@tauri-apps/api/tauri';

    let onChangeCreepScore = (value: number) => {
        invoke('set_creep_score', { value });
    };





</script>

<div class="container">
    <Slider name={"Creep score"} min={10} max={5000} step={10} value={20} onChange={onChangeCreepScore}></Slider>
    <Slider name={"Gold"} min={10} max={15000} step={1} value={200} onChange={onChangeCreepScore}></Slider>
</div>