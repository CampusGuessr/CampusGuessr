import {LeafletMap, TileLayer} from 'leaflet'
export function initMap(){

    const map = new LeafletMap('map').setView([48.1505, 11.5810], 18);

    new TileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
            maxZoom: 19,
            attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
            }).addTo(map)
return map;
}
