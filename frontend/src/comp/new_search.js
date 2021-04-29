import React, { Component } from 'react';
import { ReactiveBase, ReactiveList } from '@appbaseio/reactivesearch';

class SearchPage2 extends Component {
    render() {
        return (
            <ReactiveBase
                app="oas_feed2"
                url="http://localhost:9200"
            >
                <ReactiveList
                    dataField="dateModified"
                    componentId="SearchResult"
                    renderItem={(res) => <div key={res.identifier}>{res.headline}</div>}/>
            </ReactiveBase>
        );
    }
}


export default SearchPage2