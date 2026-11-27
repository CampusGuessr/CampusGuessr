import {Map as LeafletMap, TileLayer, LatLngBounds} from 'leaflet'
export function initMap(){
    const bounds = new LatLngBounds([47.90, 10.77],[48.46, 12.25]); //Test cords, SW NO

    const map = new LeafletMap('map',{
        center: [48.1505, 11.5810],
        zoom: 12,
        maxZoom: 19,
        minZoom: 10,

        bounceAtZoomLimits: false,
        zoomControl: false,

        maxBounds: bounds,
        maxBoundsViscosity: 1.0,
        
    });
    
    

    new TileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
            maxZoom: 19,
            attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
            }).addTo(map)
return map;
}
