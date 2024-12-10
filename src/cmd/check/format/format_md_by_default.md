<table>
    <thead>
        <tr>
            <th align="center">Status</th>
            <th align="left">Metric</th>
            <th align="left">Tags</th>
            <th align="right">Previous value</th>
            <th align="right">Current value</th>
            <th align="right">Change</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td align="center">🔴</td>
            <td>first</td>
            <td>platform.os="linux"<br />platform.arch="amd64"<br />unit="byte"</td>
            <td align="right">10.00</td>
            <td align="right">20.00</td>
            <td align="right">+10.00<br/>(+100.00 %)</td>
        </tr>
        <tr>
            <td align="center">🔴</td>
            <td colspan="5"><i>"show_not_increase_too_much" matching tags {platform.os="linux"}</i><br />increase should be less than 20.00 %</td>
        </tr>
        <tr>
            <td align="center">🟢</td>
            <td>first</td>
            <td>platform.os="linux"<br />platform.arch="arm64"<br />unit="byte"</td>
            <td align="right">10.00</td>
            <td align="right">11.00</td>
            <td align="right">+1.00<br/>(+10.00 %)</td>
        </tr>
        <tr>
            <td align="center">🟡</td>
            <td>unknown</td>
            <td></td>
            <td align="right">42.00</td>
            <td align="right">28.00</td>
            <td align="right">-14.00<br/>(-33.33 %)</td>
        </tr>
        <tr>
            <td align="center">🟡</td>
            <td>noglobal</td>
            <td></td>
            <td align="right">42.00</td>
            <td align="right">28.00</td>
            <td align="right">-14.00<br/>(-33.33 %)</td>
        </tr>
        <tr>
            <td align="center">🟢</td>
            <td>nochange</td>
            <td></td>
            <td align="right">10.00</td>
            <td align="right">10.00</td>
            <td align="right"></td>
        </tr>
        <tr>
            <td align="center">🟢</td>
            <td>with-unit</td>
            <td></td>
            <td align="right">20.00 MiB</td>
            <td align="right">25.00 MiB</td>
            <td align="right">+5.00 MiB<br/>(+25.00 %)</td>
        </tr>
        <tr>
            <td align="center">🔴</td>
            <td>with-change</td>
            <td></td>
            <td align="right">20971520.00</td>
            <td align="right">26214400.00</td>
            <td align="right">+5242880.00<br/>(+25.00 %)</td>
        </tr>
        <tr>
            <td align="center">🔴</td>
            <td colspan="5">increase should be less than 2097152.00</td>
        </tr>
    </tbody>
</table>
