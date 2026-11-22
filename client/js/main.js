import { initMap } from './map/map.js'
import { initMarker, getMarkerLatLng } from './map/markers.js'

const myMap = initMap();
const map = document.getElementById('map');
const photo = document.getElementById('photo');

initMarker(myMap);

function swapPanels(){
    map.classList.toggle('big');
    map.classList.toggle('small')

    photo.classList.toggle('big');
    photo.classList.toggle('small')
}

map.addEventListener('click', () => {
    if(map.classList.contains('small')){ 
        swapPanels();
        myMap.invalidateSize();
    }
})

photo.addEventListener('click' , () => {
    if(photo.classList.contains('small')){
        swapPanels();
        myMap.invalidateSize();
    }
})

const img = document.querySelector('#photo img')
img.src ='assets/pinny.png';