import {Icon, Marker} from 'leaflet'
let singleMarker;

export function initMarker(map){

    const pinnyIcon = new Icon({        
        iconUrl: 'assets/pinny.png',
        iconSize:     [38, 57], 
        iconAnchor:   [19, 57], 
    })

    map.on('click', function(e){
        if(singleMarker){
            singleMarker.setLatLng(e.latlng)
        } else{
            singleMarker = new Marker((e.latlng),{icon: pinnyIcon}).addTo(map)
        }
    });
}

export function getMarkerLatLng(){
    if(singleMarker){
        return singleMarker.getLatLng();
    } else {
        return null;
    }
}