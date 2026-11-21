// An Autofill service of the tests' own (032): what the system asks a password manager when the
// cursor is put in a form's field. It says what it was asked to the test, and offers nothing,
// so the system shows nothing.
//
// For a check by hand it does offer something: to a form whose field for a name is called
// "offered" (the test site's sign-in form, asked for with ?name=offered), a name and a password
// of the tests' own. The system then shows its list of what is offered, which no test brings up.
//
// In Java, and with nothing but the system's own classes: the system starts it in the tests'
// own package, which is not the app's and has no Kotlin of its own (the tests borrow the app's,
// in the app's process). It tells the test by a broadcast to the app's package alone.
package com.tigercodes.webtiger;

import android.app.assist.AssistStructure;
import android.content.Intent;
import android.os.CancellationSignal;
import android.service.autofill.AutofillService;
import android.service.autofill.Dataset;
import android.service.autofill.FillCallback;
import android.service.autofill.FillContext;
import android.service.autofill.FillRequest;
import android.service.autofill.FillResponse;
import android.service.autofill.SaveCallback;
import android.service.autofill.SaveRequest;
import android.util.Pair;
import android.view.ViewStructure;
import android.view.autofill.AutofillId;
import android.view.autofill.AutofillValue;
import android.widget.RemoteViews;

import java.util.ArrayList;
import java.util.List;

public final class AskedAutofillService extends AutofillService {
    /** What the service tells the test: that it was asked, and about what. */
    public static final String ASKED = "com.tigercodes.webtiger.test.AUTOFILL_ASKED";
    public static final String DOMAIN = "domain";
    public static final String SCHEME = "scheme";
    public static final String FIELDS = "fields";

    /** The service's name, as the device's settings name the Autofill service in use. */
    public static final String COMPONENT = "com.tigercodes.webtiger.test/com.tigercodes.webtiger.AskedAutofillService";

    /** What a form is offered when it asks for it by its field's name: for a check by hand. */
    public static final String OFFERED_FIELD = "offered";
    public static final String OFFERED_NAME = "ada";
    public static final String OFFERED_PASSWORD = "lovelace";

    private String domain = "";
    private String scheme = "";
    private AutofillId offeredName;
    private AutofillId password;

    @Override
    public void onFillRequest(FillRequest request, CancellationSignal cancellationSignal, FillCallback callback) {
        List<FillContext> contexts = request.getFillContexts();
        AssistStructure structure = contexts.get(contexts.size() - 1).getStructure();
        domain = "";
        scheme = "";
        offeredName = null;
        password = null;
        ArrayList<String> fields = new ArrayList<>();
        for (int index = 0; index < structure.getWindowNodeCount(); index++) {
            walk(structure.getWindowNodeAt(index).getRootViewNode(), fields);
        }
        sendBroadcast(
                new Intent(ASKED)
                        .setPackage(structure.getActivityComponent().getPackageName())
                        .putExtra(DOMAIN, domain)
                        .putExtra(SCHEME, scheme)
                        .putStringArrayListExtra(FIELDS, fields));
        if (offeredName != null && password != null) {
            // By hand only: one thing to pick, which fills both fields.
            RemoteViews shown = new RemoteViews(getPackageName(), android.R.layout.simple_list_item_1);
            shown.setTextViewText(android.R.id.text1, OFFERED_NAME + " (WebTiger tests)");
            Dataset both =
                    new Dataset.Builder()
                            .setValue(offeredName, AutofillValue.forText(OFFERED_NAME), shown)
                            .setValue(password, AutofillValue.forText(OFFERED_PASSWORD), shown)
                            .build();
            callback.onSuccess(new FillResponse.Builder().addDataset(both).build());
            return;
        }
        // Nothing is offered: the system has nothing to show.
        callback.onSuccess(null);
    }

    /**
     * What the page says each of its fields is, what the WebView makes of that, and whether the
     * cursor is in it. Never what is typed in it.
     */
    private void walk(AssistStructure.ViewNode node, ArrayList<String> fields) {
        if (node.getWebDomain() != null) {
            domain = node.getWebDomain();
            scheme = node.getWebScheme() == null ? "" : node.getWebScheme();
        }
        ViewStructure.HtmlInfo html = node.getHtmlInfo();
        if (html != null && "input".equalsIgnoreCase(html.getTag())) {
            String name = "";
            String type = "";
            List<Pair<String, String>> said = html.getAttributes();
            if (said != null) {
                for (Pair<String, String> one : said) {
                    if ("name".equals(one.first)) {
                        name = one.second;
                    } else if ("type".equals(one.first)) {
                        type = one.second;
                    }
                }
            }
            if (OFFERED_FIELD.equals(name)) {
                offeredName = node.getAutofillId();
            } else if ("password".equals(type)) {
                password = node.getAutofillId();
            }
            String[] hints = node.getAutofillHints();
            String hinted = hints == null ? "" : String.join("+", hints);
            fields.add(name + "/" + type + "/" + hinted + "/" + (node.isFocused() ? "focused" : ""));
        }
        for (int index = 0; index < node.getChildCount(); index++) {
            walk(node.getChildAt(index), fields);
        }
    }

    @Override
    public void onSaveRequest(SaveRequest request, SaveCallback callback) {
        callback.onSuccess();
    }
}
